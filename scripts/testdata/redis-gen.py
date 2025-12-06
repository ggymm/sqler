#!/usr/bin/env python3
"""Redis 测试数据生成脚本

功能：
    生成包含所有 Redis 数据类型的测试数据，用于测试和开发。

支持的数据类型：
    - String: 基础键值对、大 Value 测试
    - Hash: 对象存储（用户、商品、订单）
    - List: 队列和列表（消息、任务、历史）
    - Set: 集合（标签、收藏、关注）
    - Sorted Set: 排行榜（积分、销量、评分）
    - Bitmap: 签到记录
    - HyperLogLog: UV 统计
    - Geo: 地理位置
    - Stream: 事件流

输出文件：
    scripts/testdata/redis/init.redis
"""

from __future__ import annotations

import random
from datetime import datetime, timedelta
from pathlib import Path


# 设置随机种子，确保每次生成的数据一致
random.seed(42)

# 数据量配置
NUM_USERS = 5000  # 用户数量
NUM_PRODUCTS = 2000  # 商品数量
NUM_ORDERS = 3000  # 订单数量
NUM_MESSAGES = 1000  # 消息数量
NUM_SESSIONS = 500  # 会话数量
NUM_LOCATIONS = 100  # 地理位置数量

# 输出目录配置
OUTPUT_DIR = Path(__file__).parent / "redis"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)


def write_redis(filename: str, commands: list[str]) -> None:
    """写入 Redis 命令到文件

    Args:
        filename: 输出文件名
        commands: Redis 命令列表
    """
    filepath = OUTPUT_DIR / filename
    with filepath.open("w", encoding="utf-8") as fp:
        fp.write("\n".join(commands))
    print(f"  ✅ {filename} ({len(commands):,} 条命令)")


def generate_strings() -> list[str]:
    """生成 String 类型数据

    包含：
        - 用户基础信息（姓名、年龄、邮箱、手机、状态）
        - 商品价格和库存信息
        - 会话令牌（带过期时间）
        - 全局计数器
        - 大 Value 测试数据（JSON 文档 ~10KB）
        - 超大 Value 测试数据（Blob ~100KB）

    Returns:
        Redis SET/SETEX 命令列表
    """
    print("  [String] 生成用户基础信息...")
    commands = []
    names_cn = ["张伟", "李娜", "王芳", "刘强", "陈静", "杨磊", "赵敏", "孙丽", "周杰", "吴勇"]

    # 用户基础信息：每个用户 5 个字段
    for i in range(1, NUM_USERS + 1):
        commands.append(f'SET user:{i}:name "{random.choice(names_cn)}{i}"')
        commands.append(f"SET user:{i}:age {random.randint(18, 65)}")
        commands.append(f'SET user:{i}:email "user{i}@example.com"')
        commands.append(
            f'SET user:{i}:phone "+86-{random.randint(13000000000, 18999999999)}"'
        )
        commands.append(
            f'SET user:{i}:status "{random.choice(["active", "inactive", "banned"])}"'
        )

    print(f"    ✓ 用户信息: {NUM_USERS * 5:,} 条")

    # 商品价格和库存：每个商品 3 个字段
    print("  [String] 生成商品价格和库存...")
    for i in range(1, NUM_PRODUCTS + 1):
        commands.append(f"SET product:{i}:price {random.uniform(10, 9999):.2f}")
        commands.append(f"SET product:{i}:stock {random.randint(0, 10000)}")
        commands.append(f'SET product:{i}:sku "SKU{i:06d}"')

    print(f"    ✓ 商品信息: {NUM_PRODUCTS * 3:,} 条")

    # 会话令牌：带 1 小时过期时间
    print("  [String] 生成会话令牌...")
    for i in range(1, NUM_SESSIONS + 1):
        token = f"token_{random.randint(100000, 999999)}_{i}"
        commands.append(
            f'SETEX session:{token} 3600 "user_id:{random.randint(1, NUM_USERS)}"'
        )

    print(f"    ✓ 会话令牌: {NUM_SESSIONS:,} 条")

    # 全局计数器
    print("  [String] 生成全局计数器...")
    commands.append(f"SET counter:page_views {random.randint(1000000, 9999999)}")
    commands.append(f"SET counter:total_users {NUM_USERS}")
    commands.append(f"SET counter:total_orders {NUM_ORDERS}")
    commands.append(f"SET counter:daily_sales {random.randint(10000, 99999)}")
    print(f"    ✓ 计数器: 4 条")

    # 大 Value 测试：JSON 文档（约 10KB）
    print("  [String] 生成大 Value 测试数据 (~10KB JSON)...")
    for i in range(1, 51):
        large_json = "{"
        large_json += f'"id":{i},'
        large_json += f'"title":"大型JSON文档测试 {i}",'
        large_json += (
            '"description":"'
            + "这是一个非常长的描述信息，用于测试大 value 的存储和读取性能。" * 50
            + '",'
        )
        large_json += '"tags":["tag1","tag2","tag3","tag4","tag5"],'
        large_json += (
            '"metadata":{"created_at":"2024-01-01T00:00:00Z",'
            '"updated_at":"2024-12-06T00:00:00Z"},'
        )
        large_json += (
            '"content":"'
            + "Lorem ipsum dolor sit amet, consectetur adipiscing elit. " * 100
            + '"'
        )
        large_json += "}"
        commands.append(f"SET document:large:{i} '{large_json}'")

    print(f"    ✓ 大 JSON: 50 条 (~500KB)")

    # 超大 Value 测试：纯文本 Blob（100KB）
    print("  [String] 生成超大 Value 测试数据 (~100KB Blob)...")
    for i in range(1, 11):
        huge_content = "A" * 102400  # 100KB
        commands.append(f'SET blob:huge:{i} "{huge_content}"')

    print(f"    ✓ 超大 Blob: 10 条 (~1MB)")

    return commands


def generate_hashes() -> list[str]:
    """生成 Hash 类型数据

    包含：
        - 用户详细信息（用户名、城市、等级等）
        - 商品详细信息（名称、分类、品牌等）
        - 订单信息
        - 购物车数据

    Returns:
        Redis HSET 命令列表
    """
    print("  [Hash] 生成用户详细信息...")
    commands = []
    cities = [
        "北京",
        "上海",
        "广州",
        "深圳",
        "杭州",
        "成都",
        "武汉",
        "西安",
        "南京",
        "重庆",
    ]

    # 用户详细信息：每个用户一个 Hash
    for i in range(1, NUM_USERS + 1):
        commands.append(
            f'HSET user:detail:{i} '
            f'id {i} '
            f'username "user{i}" '
            f'nickname "昵称{i}" '
            f'city "{random.choice(cities)}" '
            f'gender "{random.choice(["male", "female", "other"])}" '
            f'level {random.randint(1, 100)} '
            f'vip "{random.choice(["true", "false"])}" '
            f'balance {random.uniform(0, 10000):.2f} '
            f'created_at "{datetime.now() - timedelta(days=random.randint(1, 365))}"'
        )

    print(f"    ✓ 用户详情: {NUM_USERS:,} 条")

    # 商品详细信息
    print("  [Hash] 生成商品详细信息...")
    categories = [
        "电子产品",
        "服装鞋包",
        "食品饮料",
        "家居用品",
        "图书音像",
        "运动户外",
        "美妆个护",
        "母婴玩具",
    ]
    brands = [
        "Apple",
        "Samsung",
        "Nike",
        "Adidas",
        "Sony",
        "LG",
        "Huawei",
        "Xiaomi",
    ]

    for i in range(1, NUM_PRODUCTS + 1):
        commands.append(
            f'HSET product:detail:{i} '
            f'id {i} '
            f'name "商品名称{i}" '
            f'category "{random.choice(categories)}" '
            f'brand "{random.choice(brands)}" '
            f'price {random.uniform(10, 9999):.2f} '
            f'stock {random.randint(0, 10000)} '
            f'sales {random.randint(0, 50000)} '
            f'rating {random.uniform(3.5, 5.0):.1f} '
            f'description "这是商品{i}的详细描述信息"'
        )

    print(f"    ✓ 商品详情: {NUM_PRODUCTS:,} 条")

    # 订单信息
    print("  [Hash] 生成订单信息...")
    statuses = ["pending", "paid", "shipped", "delivered", "cancelled"]
    for i in range(1, NUM_ORDERS + 1):
        commands.append(
            f'HSET order:{i} '
            f'order_id {i} '
            f'user_id {random.randint(1, NUM_USERS)} '
            f'product_id {random.randint(1, NUM_PRODUCTS)} '
            f'quantity {random.randint(1, 10)} '
            f'total_amount {random.uniform(10, 9999):.2f} '
            f'status "{random.choice(statuses)}" '
            f'created_at "{datetime.now() - timedelta(days=random.randint(1, 90))}" '
            f'payment_method "{random.choice(["alipay", "wechat", "credit_card"])}"'
        )

    print(f"    ✓ 订单: {NUM_ORDERS:,} 条")

    # 购物车
    print("  [Hash] 生成购物车数据...")
    cart_count = 500
    for i in range(1, cart_count + 1):
        num_items = random.randint(1, 8)
        fields = []
        for _ in range(num_items):
            product_id = random.randint(1, NUM_PRODUCTS)
            quantity = random.randint(1, 5)
            fields.append(f"product:{product_id} {quantity}")
        commands.append(f'HSET cart:user:{i} {" ".join(fields)}')

    print(f"    ✓ 购物车: {cart_count} 条")

    return commands


def generate_lists() -> list[str]:
    """生成 List 类型数据

    包含：
        - 消息队列（JSON 格式消息）
        - 任务队列
        - 用户最近订单（LTRIM 限制长度）
        - 浏览历史
        - 通知列表

    Returns:
        Redis LPUSH/RPUSH/LTRIM 命令列表
    """
    print("  [List] 生成消息队列...")
    commands = []

    # 消息队列
    for i in range(1, NUM_MESSAGES + 1):
        msg_types = ["email", "sms", "push", "webhook"]
        commands.append(
            f'LPUSH queue:messages "{{\\"id\\":{i},\\"type\\":\\"{random.choice(msg_types)}\\",\\"user_id\\":{random.randint(1, NUM_USERS)}}}"'
        )

    print(f"    ✓ 消息队列: {NUM_MESSAGES:,} 条")

    # 任务队列
    print("  [List] 生成任务队列...")
    task_count = 1000
    for i in range(1, task_count + 1):
        task_types = [
            "report_generate",
            "email_send",
            "data_backup",
            "image_process",
        ]
        commands.append(f'RPUSH queue:tasks "task:{random.choice(task_types)}:{i}"')

    print(f"    ✓ 任务队列: {task_count:,} 条")

    # 用户最近订单（每人最多保留 20 条）
    print("  [List] 生成用户最近订单...")
    order_list_users = 1000
    order_commands = 0
    for user_id in range(1, order_list_users + 1):
        num_orders = random.randint(1, 20)
        for _ in range(num_orders):
            order_id = random.randint(1, NUM_ORDERS)
            commands.append(f"LPUSH user:{user_id}:recent_orders {order_id}")
            order_commands += 1
        commands.append(f"LTRIM user:{user_id}:recent_orders 0 19")

    print(f"    ✓ 最近订单: {order_list_users:,} 个用户, {order_commands:,} 条记录")

    # 浏览历史
    print("  [List] 生成浏览历史...")
    browse_users = 500
    browse_commands = 0
    for user_id in range(1, browse_users + 1):
        num_views = random.randint(10, 50)
        for _ in range(num_views):
            product_id = random.randint(1, NUM_PRODUCTS)
            commands.append(f"LPUSH user:{user_id}:browse_history {product_id}")
            browse_commands += 1
        commands.append(f"LTRIM user:{user_id}:browse_history 0 99")

    print(f"    ✓ 浏览历史: {browse_users} 个用户, {browse_commands:,} 条记录")

    # 通知列表
    print("  [List] 生成通知列表...")
    notifications = [
        "您的订单已发货",
        "您有新的消息",
        "系统维护通知",
        "账户安全提醒",
        "优惠券即将过期",
        "新品上架通知",
    ]
    notif_users = 1000
    notif_commands = 0
    for user_id in range(1, notif_users + 1):
        num_notif = random.randint(3, 15)
        for _ in range(num_notif):
            commands.append(
                f'LPUSH user:{user_id}:notifications "{random.choice(notifications)}"'
            )
            notif_commands += 1

    print(f"    ✓ 通知: {notif_users:,} 个用户, {notif_commands:,} 条通知")

    return commands


def generate_sets() -> list[str]:
    """生成 Set 类型数据

    包含：
        - 商品标签集合
        - 用户收藏集合
        - 在线用户集合
        - 用户关注/粉丝关系
        - 分类下的商品集合

    Returns:
        Redis SADD 命令列表
    """
    print("  [Set] 生成商品标签...")
    commands = []
    all_tags = [
        "热销",
        "新品",
        "特价",
        "包邮",
        "进口",
        "限时",
        "精选",
        "推荐",
        "高端",
        "性价比",
        "品质",
        "畅销",
        "口碑",
        "优惠",
        "折扣",
    ]

    # 商品标签：每个商品 2-6 个标签
    for i in range(1, NUM_PRODUCTS + 1):
        num_tags = random.randint(2, 6)
        tags = random.sample(all_tags, num_tags)
        tags_str = " ".join(f'"{tag}"' for tag in tags)
        commands.append(f"SADD product:{i}:tags {tags_str}")

    print(f"    ✓ 商品标签: {NUM_PRODUCTS:,} 个商品")

    # 用户收藏
    print("  [Set] 生成用户收藏...")
    fav_users = 2000
    for user_id in range(1, fav_users + 1):
        num_fav = random.randint(5, 30)
        products = random.sample(range(1, NUM_PRODUCTS + 1), num_fav)
        commands.append(f'SADD user:{user_id}:favorites {" ".join(map(str, products))}')

    print(f"    ✓ 用户收藏: {fav_users:,} 个用户")

    # 在线用户
    print("  [Set] 生成在线用户...")
    online_users = random.sample(range(1, NUM_USERS + 1), 500)
    commands.append(f'SADD online_users {" ".join(map(str, online_users))}')
    print(f"    ✓ 在线用户: 500 人")

    # 用户关注/粉丝关系
    print("  [Set] 生成用户关注关系...")
    follow_users = 1000
    for user_id in range(1, follow_users + 1):
        # 关注的人
        num_following = random.randint(10, 100)
        following = random.sample(range(1, NUM_USERS + 1), num_following)
        commands.append(
            f'SADD user:{user_id}:following {" ".join(map(str, following))}'
        )

        # 粉丝
        num_followers = random.randint(5, 200)
        followers = random.sample(range(1, NUM_USERS + 1), num_followers)
        commands.append(
            f'SADD user:{user_id}:followers {" ".join(map(str, followers))}'
        )

    print(f"    ✓ 关注关系: {follow_users:,} 个用户")

    # 分类下的商品
    print("  [Set] 生成分类商品集合...")
    categories = ["电子产品", "服装鞋包", "食品饮料", "家居用品", "图书音像"]
    for cat in categories:
        num_prods = random.randint(100, 400)
        products = random.sample(range(1, NUM_PRODUCTS + 1), num_prods)
        commands.append(
            f'SADD category:"{cat}":products {" ".join(map(str, products))}'
        )

    print(f"    ✓ 分类商品: {len(categories)} 个分类")

    return commands


def generate_sorted_sets() -> list[str]:
    """生成 Sorted Set (ZSet) 类型数据

    包含：
        - 用户积分排行榜
        - 商品销量排行榜
        - 商品评分排行榜
        - 热门搜索词排行
        - 用户活跃度排行
        - 事件时间序列

    Returns:
        Redis ZADD 命令列表
    """
    print("  [Sorted Set] 生成用户积分排行...")
    commands = []

    # 用户积分排行
    for user_id in range(1, NUM_USERS + 1):
        score = random.randint(0, 100000)
        commands.append(f"ZADD leaderboard:points {score} user:{user_id}")

    print(f"    ✓ 积分排行: {NUM_USERS:,} 个用户")

    # 商品销量排行
    print("  [Sorted Set] 生成商品销量排行...")
    for product_id in range(1, NUM_PRODUCTS + 1):
        sales = random.randint(0, 50000)
        commands.append(f"ZADD leaderboard:sales {sales} product:{product_id}")

    print(f"    ✓ 销量排行: {NUM_PRODUCTS:,} 个商品")

    # 商品评分排行
    print("  [Sorted Set] 生成商品评分排行...")
    for product_id in range(1, NUM_PRODUCTS + 1):
        rating = random.uniform(3.0, 5.0)
        commands.append(f"ZADD leaderboard:rating {rating:.2f} product:{product_id}")

    print(f"    ✓ 评分排行: {NUM_PRODUCTS:,} 个商品")

    # 热门搜索词
    print("  [Sorted Set] 生成热门搜索...")
    keywords = [
        "iPhone",
        "MacBook",
        "iPad",
        "AirPods",
        "手机",
        "电脑",
        "耳机",
        "鼠标",
        "键盘",
        "显示器",
        "充电器",
        "数据线",
        "保护套",
        "钢化膜",
        "移动电源",
        "蓝牙音箱",
        "智能手表",
        "运动手环",
        "平板电脑",
        "游戏机",
    ]
    for keyword in keywords:
        count = random.randint(100, 50000)
        commands.append(f'ZADD trending:searches {count} "{keyword}"')

    print(f"    ✓ 热搜词: {len(keywords)} 个")

    # 用户活跃度排行
    print("  [Sorted Set] 生成活跃度排行...")
    activity_users = 2000
    for user_id in range(1, activity_users + 1):
        activity_score = random.randint(0, 10000)
        commands.append(f"ZADD leaderboard:activity {activity_score} user:{user_id}")

    print(f"    ✓ 活跃度: {activity_users:,} 个用户")

    # 时间序列事件
    print("  [Sorted Set] 生成事件时间序列...")
    event_count = 1000
    base_time = int(datetime.now().timestamp())
    for i in range(1, event_count + 1):
        timestamp = base_time - random.randint(0, 86400 * 30)  # 最近30天
        commands.append(f'ZADD events:timeline {timestamp} "event:{i}"')

    print(f"    ✓ 事件序列: {event_count:,} 条")

    return commands


def generate_bitmaps() -> list[str]:
    """生成 Bitmap 类型数据

    包含：
        - 用户签到记录（最近 30 天）

    Returns:
        Redis SETBIT 命令列表
    """
    print("  [Bitmap] 生成用户签到记录...")
    commands = []
    base_date = datetime.now()
    signin_users = 1000
    total_bits = 0

    # 最近 30 天的签到记录
    for day_offset in range(30):
        date = base_date - timedelta(days=day_offset)
        date_str = date.strftime("%Y%m%d")
        # 每天 30%-70% 的用户签到
        num_signin = random.randint(300, 700)
        signin_list = random.sample(range(1, signin_users + 1), num_signin)
        for user_id in signin_list:
            commands.append(f"SETBIT user:signin:{date_str} {user_id} 1")
            total_bits += 1

    print(f"    ✓ 签到记录: 30 天, {signin_users} 个用户, {total_bits:,} 条记录")

    return commands


def generate_hyperloglogs() -> list[str]:
    """生成 HyperLogLog 类型数据

    包含：
        - 每日 UV 统计（最近 30 天）
        - 页面 UV 统计

    Returns:
        Redis PFADD 命令列表
    """
    print("  [HyperLogLog] 生成每日 UV...")
    commands = []
    base_date = datetime.now()
    uv_commands = 0

    # 每日 UV 统计
    for day_offset in range(30):
        date = base_date - timedelta(days=day_offset)
        date_str = date.strftime("%Y%m%d")
        num_visitors = random.randint(1000, 3000)
        visitors = [
            f"user:{random.randint(1, NUM_USERS)}" for _ in range(num_visitors)
        ]
        # 分批添加（每次最多 100 个）
        for i in range(0, len(visitors), 100):
            batch = visitors[i : i + 100]
            commands.append(f'PFADD uv:daily:{date_str} {" ".join(batch)}')
            uv_commands += 1

    print(f"    ✓ 每日 UV: 30 天, {uv_commands:,} 批次")

    # 页面 UV 统计
    print("  [HyperLogLog] 生成页面 UV...")
    pages = [
        "home",
        "product_list",
        "product_detail",
        "cart",
        "checkout",
        "user_center",
    ]
    page_commands = 0
    for page in pages:
        num_visitors = random.randint(500, 2000)
        visitors = [
            f"user:{random.randint(1, NUM_USERS)}" for _ in range(num_visitors)
        ]
        for i in range(0, len(visitors), 100):
            batch = visitors[i : i + 100]
            commands.append(f'PFADD uv:page:{page} {" ".join(batch)}')
            page_commands += 1

    print(f"    ✓ 页面 UV: {len(pages)} 个页面, {page_commands:,} 批次")

    return commands


def generate_geos() -> list[str]:
    """生成 Geo 类型数据

    包含：
        - 门店地理位置
        - 快递员实时位置

    Returns:
        Redis GEOADD 命令列表
    """
    print("  [Geo] 生成门店位置...")
    commands = []
    # 中国主要城市坐标
    cities_coords = [
        ("北京", 116.404, 39.915),
        ("上海", 121.472, 31.231),
        ("广州", 113.264, 23.129),
        ("深圳", 114.057, 22.543),
        ("杭州", 120.153, 30.287),
        ("成都", 104.066, 30.572),
        ("武汉", 114.305, 30.593),
        ("西安", 108.940, 34.341),
        ("南京", 118.796, 32.059),
        ("重庆", 106.551, 29.563),
    ]

    # 门店位置：在城市周边随机分布
    for i in range(1, NUM_LOCATIONS + 1):
        _city, base_lng, base_lat = random.choice(cities_coords)
        lng = base_lng + random.uniform(-0.5, 0.5)  # 约 50km 范围
        lat = base_lat + random.uniform(-0.5, 0.5)
        commands.append(f'GEOADD stores {lng:.6f} {lat:.6f} "store:{i}"')

    print(f"    ✓ 门店: {NUM_LOCATIONS} 个")

    # 快递员位置
    print("  [Geo] 生成快递员位置...")
    courier_count = 200
    for i in range(1, courier_count + 1):
        _city, base_lng, base_lat = random.choice(cities_coords)
        lng = base_lng + random.uniform(-0.3, 0.3)  # 约 30km 范围
        lat = base_lat + random.uniform(-0.3, 0.3)
        commands.append(f'GEOADD couriers {lng:.6f} {lat:.6f} "courier:{i}"')

    print(f"    ✓ 快递员: {courier_count} 个")

    return commands


def generate_streams() -> list[str]:
    """生成 Stream 类型数据

    包含：
        - 订单事件流
        - 用户行为事件流
        - 系统日志流

    Returns:
        Redis XADD 命令列表
    """
    print("  [Stream] 生成订单事件流...")
    commands = []
    actions = ["created", "paid", "shipped", "delivered", "cancelled"]
    order_events = 1000

    # 订单事件流
    for i in range(1, order_events + 1):
        order_id = random.randint(1, NUM_ORDERS)
        user_id = random.randint(1, NUM_USERS)
        action = random.choice(actions)
        amount = random.uniform(10, 9999)
        commands.append(
            f"XADD stream:orders * "
            f"order_id {order_id} "
            f"user_id {user_id} "
            f"action {action} "
            f"amount {amount:.2f}"
        )

    print(f"    ✓ 订单事件: {order_events:,} 条")

    # 用户行为事件流
    print("  [Stream] 生成用户行为事件...")
    actions = ["view", "click", "add_to_cart", "purchase", "share", "comment"]
    user_events = 2000
    for i in range(1, user_events + 1):
        user_id = random.randint(1, NUM_USERS)
        product_id = random.randint(1, NUM_PRODUCTS)
        action = random.choice(actions)
        commands.append(
            f"XADD stream:user_actions * "
            f"user_id {user_id} "
            f"product_id {product_id} "
            f"action {action} "
            f"timestamp {int(datetime.now().timestamp())}"
        )

    print(f"    ✓ 用户行为: {user_events:,} 条")

    # 系统日志流
    print("  [Stream] 生成系统日志...")
    levels = ["INFO", "WARN", "ERROR"]
    modules = ["auth", "order", "payment", "shipping", "notification"]
    log_events = 500
    for i in range(1, log_events + 1):
        level = random.choice(levels)
        module = random.choice(modules)
        commands.append(
            f"XADD stream:logs * "
            f"level {level} "
            f"module {module} "
            f'message "Log_message_{i}" '
            f"timestamp {int(datetime.now().timestamp())}"
        )

    print(f"    ✓ 系统日志: {log_events:,} 条")

    return commands


def main() -> None:
    """主函数：生成所有类型的 Redis 测试数据"""
    print("=" * 60)
    print("Redis 测试数据生成")
    print("=" * 60)
    print(f"输出目录: {OUTPUT_DIR}")
    print(f"输出文件: {OUTPUT_DIR / 'init.redis'}")
    print()
    print("数据量配置:")
    print(f"  用户数: {NUM_USERS:,}")
    print(f"  商品数: {NUM_PRODUCTS:,}")
    print(f"  订单数: {NUM_ORDERS:,}")
    print(f"  消息数: {NUM_MESSAGES:,}")
    print(f"  会话数: {NUM_SESSIONS:,}")
    print(f"  位置数: {NUM_LOCATIONS:,}")
    print()

    # 生成各类型数据
    print("开始生成数据...")
    print()

    print("[1/9] String 类型")
    strings = generate_strings()
    print(f"  总计: {len(strings):,} 条命令")
    print()

    print("[2/9] Hash 类型")
    hashes = generate_hashes()
    print(f"  总计: {len(hashes):,} 条命令")
    print()

    print("[3/9] List 类型")
    lists = generate_lists()
    print(f"  总计: {len(lists):,} 条命令")
    print()

    print("[4/9] Set 类型")
    sets = generate_sets()
    print(f"  总计: {len(sets):,} 条命令")
    print()

    print("[5/9] Sorted Set 类型")
    sorted_sets = generate_sorted_sets()
    print(f"  总计: {len(sorted_sets):,} 条命令")
    print()

    print("[6/9] Bitmap 类型")
    bitmaps = generate_bitmaps()
    print(f"  总计: {len(bitmaps):,} 条命令")
    print()

    print("[7/9] HyperLogLog 类型")
    hyperloglogs = generate_hyperloglogs()
    print(f"  总计: {len(hyperloglogs):,} 条命令")
    print()

    print("[8/9] Geo 类型")
    geos = generate_geos()
    print(f"  总计: {len(geos):,} 条命令")
    print()

    print("[9/9] Stream 类型")
    streams = generate_streams()
    print(f"  总计: {len(streams):,} 条命令")
    print()

    # 合并所有命令
    print("合并所有命令...")
    all_commands = (
        strings
        + hashes
        + lists
        + sets
        + sorted_sets
        + bitmaps
        + hyperloglogs
        + geos
        + streams
    )
    print(f"  命令总数: {len(all_commands):,}")
    print()

    # 写入文件
    print("写入文件...")
    write_redis("init.redis", all_commands)
    print()

    # 统计信息
    print("=" * 60)
    print("✅ 生成完成!")
    print("=" * 60)
    print("数据统计:")
    print(f"  String:       {len(strings):>8,} 条")
    print(f"  Hash:         {len(hashes):>8,} 条")
    print(f"  List:         {len(lists):>8,} 条")
    print(f"  Set:          {len(sets):>8,} 条")
    print(f"  Sorted Set:   {len(sorted_sets):>8,} 条")
    print(f"  Bitmap:       {len(bitmaps):>8,} 条")
    print(f"  HyperLogLog:  {len(hyperloglogs):>8,} 条")
    print(f"  Geo:          {len(geos):>8,} 条")
    print(f"  Stream:       {len(streams):>8,} 条")
    print(f"  {'─' * 30}")
    print(f"  总计:         {len(all_commands):>8,} 条")
    print()
    print("导入命令:")
    print(f"  redis-cli -h localhost -p 6379 < {OUTPUT_DIR / 'init.redis'}")
    print()


if __name__ == "__main__":
    main()
