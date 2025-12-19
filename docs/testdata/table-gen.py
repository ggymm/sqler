#!/usr/bin/env python3
"""Table 测试数据生成脚本

功能：
    生成电商业务场景的关系型数据库测试数据，用于测试和开发。

包含的表：
    - customer: 客户信息
    - customer_address: 客户地址
    - category: 商品分类
    - products: 商品信息
    - product_translation: 商品多语言翻译
    - order: 订单
    - order_items: 订单明细
    - payment: 支付记录
    - shipment: 物流信息
    - product_review: 商品评价
    - support_ticket: 客服工单

输出文件：
    scripts/testdata/table/*.csv (11 个 CSV 文件)
"""

from __future__ import annotations

import csv
import random
from datetime import datetime, timedelta
from pathlib import Path


# 设置随机种子，确保每次生成的数据一致
random.seed(42)

# 数据量配置
NUM_CUSTOMERS = 1000  # 客户数量
NUM_ADDRESSES = 1000  # 地址数量
NUM_CATEGORIES = 1000  # 分类数量
NUM_PRODUCTS = 1000  # 商品数量
NUM_ORDERS = 1000  # 订单数量
NUM_ORDER_ITEMS = 3000  # 订单明细数量
NUM_PAYMENTS = 1000  # 支付记录数量
NUM_SHIPMENTS = 1000  # 物流记录数量
NUM_REVIEWS = 1000  # 评价数量
NUM_TICKETS = 1000  # 工单数量

# 输出目录配置
OUTPUT_DIR = Path(__file__).parent / "table"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)


def write_csv(filename: str, header: list[str], rows: list[list[object]]) -> None:
    """写入 CSV 文件

    Args:
        filename: 输出文件名
        header: CSV 表头
        rows: 数据行列表
    """
    filepath = OUTPUT_DIR / filename
    with filepath.open("w", newline="", encoding="utf-8") as fp:
        writer = csv.writer(fp)
        writer.writerow(header)
        writer.writerows(rows)
    print(f"  ✅ {filename:<30} {len(rows):>6,} 行")


def generate_customers() -> list[list[object]]:
    """生成客户信息

    包含字段：
        - customer_id: 客户ID
        - full_name: 全名（支持多语言）
        - email: 邮箱
        - phone: 电话
        - locale: 语言区域
        - status: 账户状态
        - created_at: 创建时间
        - loyalty_points: 积分

    Returns:
        客户数据行列表
    """
    locales = ["en-US", "zh-CN", "es-ES", "fr-FR", "ja-JP"]
    statuses = ["ACTIVE", "INACTIVE", "SUSPENDED"]
    rows = []
    base_date = datetime(2023, 1, 1, 8, 0, 0)
    family_names_cn = ["王", "李", "张", "刘", "陈", "杨", "黄", "赵"]
    given_names_cn = ["伟", "芳", "娜", "敏", "静", "丽", "强", "磊"]
    family_names_es = ["García", "Martínez", "Rodríguez", "Fernández", "López"]
    given_names_es = ["María", "José", "Luis", "Ana", "Lucía"]
    family_names_en = ["Johnson", "Brown", "Smith", "Clark", "Davis", "Moore"]
    given_names_en = ["Alice", "Bruno", "Carla", "Daniel", "Eva", "Frank"]

    # 生成客户数据
    for cid in range(1, NUM_CUSTOMERS + 1):
        locale = random.choice(locales)
        # 根据语言区域生成对应的姓名
        if locale == "zh-CN":
            full_name = random.choice(family_names_cn) + random.choice(given_names_cn)
        elif locale == "es-ES":
            full_name = (
                f"{random.choice(given_names_es)} {random.choice(family_names_es)}"
            )
        else:
            full_name = (
                f"{random.choice(given_names_en)} {random.choice(family_names_en)}"
            )
        email = f"customer{cid}@example.com"
        phone = f"+1-202-555-{cid:04d}"
        status = statuses[cid % len(statuses)]
        created_at = base_date + timedelta(minutes=cid)
        loyalty_points = (cid * 7) % 1500
        rows.append(
            [
                cid,
                full_name,
                email,
                phone,
                locale,
                status,
                created_at.isoformat(),
                loyalty_points,
            ]
        )
    return rows


def generate_addresses() -> list[list[object]]:
    """生成客户地址

    包含字段：
        - address_id: 地址ID
        - customer_id: 客户ID（外键）
        - address_type: 地址类型（账单/收货/办公/家庭）
        - line1: 地址行1
        - line2: 地址行2
        - city: 城市
        - region: 区域/州
        - postal_code: 邮编
        - country: 国家
        - latitude: 纬度
        - longitude: 经度

    Returns:
        地址数据行列表
    """
    address_types = ["billing", "shipping", "office", "home"]
    cities = [
        ("New York", "USA"),
        ("San Francisco", "USA"),
        ("北京", "中国"),
        ("上海", "中国"),
        ("Madrid", "España"),
        ("Barcelona", "España"),
        ("Toronto", "Canada"),
        ("Tokyo", "日本"),
    ]
    rows = []
    base_lat, base_lng = 40.0, -74.0
    # 生成地址数据
    for aid in range(1, NUM_ADDRESSES + 1):
        customer_id = ((aid - 1) % NUM_CUSTOMERS) + 1  # 循环分配给客户
        addr_type = random.choice(address_types)
        city, country = random.choice(cities)
        street_no = random.randint(10, 9999)
        line1 = f"{street_no} {addr_type.capitalize()} Street"
        line2 = f"Apt {random.randint(1, 500)}"
        region = f"Region-{random.randint(1, 50)}"
        postal_code = f"{random.randint(10000, 99999)}"
        latitude = round(base_lat + random.uniform(-5, 5), 6)
        longitude = round(base_lng + random.uniform(-10, 10), 6)
        rows.append(
            [
                aid,
                customer_id,
                addr_type,
                line1,
                line2,
                city,
                region,
                postal_code,
                country,
                latitude,
                longitude,
            ]
        )
    return rows


def generate_categories() -> list[list[object]]:
    """生成商品分类

    包含字段：
        - category_id: 分类ID
        - parent_category_id: 父分类ID（支持层级结构）
        - slug: URL友好的分类标识
        - display_name_en: 英文显示名
        - display_name_zh: 中文显示名
        - display_name_es: 西班牙文显示名
        - description: 描述
        - created_at: 创建时间

    Returns:
        分类数据行列表
    """
    base_names = [
        ("electronics", "Electronics", "电子产品", "Electrónica"),
        ("fashion", "Fashion", "时尚服饰", "Moda"),
        ("books", "Books", "图书", "Libros"),
        ("home-decor", "Home Decor", "家居装饰", "Decoración"),
        ("sports", "Sports", "运动户外", "Deportes"),
        ("beauty", "Beauty", "美妆护肤", "Belleza"),
        ("grocery", "Grocery", "生鲜食品", "Alimentos"),
        ("toys", "Toys", "玩具乐器", "Juguetes"),
        ("travel", "Travel", "旅行用品", "Viajes"),
        ("digital", "Digital", "数码设备", "Digitales"),
    ]
    rows = []
    base_date = datetime(2022, 5, 1, 9, 0, 0)
    # 生成分类数据
    for cid in range(1, NUM_CATEGORIES + 1):
        slug_base = random.choice(base_names)
        slug = f"{slug_base[0]}-{cid}"
        # 前50个为顶级分类，其余为子分类
        parent_id = "" if cid <= 50 else random.randint(1, 50)
        created_at = base_date + timedelta(minutes=cid)
        rows.append(
            [
                cid,
                parent_id,
                slug,
                f"{slug_base[1]} {cid}",
                f"{slug_base[2]} {cid}",
                f"{slug_base[3]} {cid}",
                f"{slug_base[1]} category {cid} description",
                created_at.isoformat(),
            ]
        )
    return rows


def generate_products(
    categories: list[list[object]],
) -> tuple[list[list[object]], list[float]]:
    """生成商品信息

    包含字段：
        - product_id: 商品ID
        - category_id: 分类ID（外键）
        - sku: 商品编号
        - price: 售价
        - cost: 成本
        - currency: 货币
        - status: 状态（在售/停售）
        - created_at: 创建时间

    Args:
        categories: 分类数据（用于获取有效的分类ID）

    Returns:
        (商品数据行列表, 价格列表)
    """
    statuses = ["ACTIVE", "DISCONTINUED"]
    currencies = ["USD", "CNY", "EUR", "JPY"]
    rows = []
    prices = []
    base_date = datetime(2022, 6, 1, 10, 0, 0)
    category_ids = [row[0] for row in categories]

    # 生成商品数据
    for pid in range(1, NUM_PRODUCTS + 1):
        category_id = random.choice(category_ids)
        sku = f"SKU{pid:05d}"  # 5位编号
        price = round(10 + (pid * 2.37) % 500, 2)  # 10-510 元
        cost = round(price * random.uniform(0.4, 0.8), 2)  # 成本为售价的40%-80%
        currency = random.choice(currencies)
        # 每11个商品中有1个停售
        status = statuses[pid % len(statuses)] if pid % 11 == 0 else "ACTIVE"
        created_at = base_date + timedelta(days=pid)
        rows.append(
            [
                pid,
                category_id,
                sku,
                price,
                cost,
                currency,
                status,
                created_at.isoformat(),
            ]
        )
        prices.append(price)
    return rows, prices


def generate_product_translations(products: list[list[object]]) -> list[list[object]]:
    """生成商品多语言翻译

    包含字段：
        - product_id: 商品ID（外键）
        - locale: 语言区域
        - name: 商品名称
        - description: 商品描述

    Args:
        products: 商品数据

    Returns:
        翻译数据行列表（每个商品3种语言）
    """
    locales = ["en-US", "zh-CN", "es-ES"]
    adjectives = {
        "en-US": ["Premium", "Eco", "Smart", "Limited", "Classic", "Ultra"],
        "zh-CN": ["旗舰版", "环保款", "智能版", "限量版", "经典款", "升级版"],
        "es-ES": [
            "Premium",
            "Eco",
            "Inteligente",
            "Edición limitada",
            "Clásico",
            "Ultra",
        ],
    }
    nouns = {
        "en-US": ["Device", "Bundle", "Kit", "Solution", "Accessory", "Package"],
        "zh-CN": ["设备", "套装", "组合", "方案", "配件", "礼包"],
        "es-ES": ["Dispositivo", "Paquete", "Kit", "Solución", "Accesorio", "Combo"],
    }
    descriptions = {
        "en-US": "Inclusive design supporting multi-language experience.",
        "zh-CN": "支持多语言体验的通用化设计。",
        "es-ES": "Diseño inclusivo con soporte multilingüe.",
    }
    rows = []
    # 为每个商品生成3种语言的翻译
    for (
        product_id,
        category_id,
        sku,
        price,
        cost,
        currency,
        status,
        created_at,
    ) in products:
        for locale in locales:
            title = f"{random.choice(adjectives[locale])} {nouns[locale][product_id % len(nouns[locale])]}"
            body = descriptions[locale]
            rows.append(
                [
                    product_id,
                    locale,
                    title,
                    body,
                ]
            )
    return rows


def generate_orders(
    customers: list[list[object]], addresses: list[list[object]]
) -> list[list[object]]:
    """生成订单

    包含字段：
        - order_id: 订单ID
        - customer_id: 客户ID（外键）
        - order_date: 下单时间
        - status: 订单状态
        - total_amount: 总金额（占位，后续更新）
        - currency: 货币
        - shipping_address_id: 收货地址ID
        - billing_address_id: 账单地址ID

    Args:
        customers: 客户数据
        addresses: 地址数据

    Returns:
        订单数据行列表
    """
    statuses = ["PENDING", "PAID", "SHIPPED", "COMPLETED", "CANCELLED"]
    currencies = ["USD", "CNY", "EUR", "JPY"]
    rows = []
    base_date = datetime(2023, 7, 1, 9, 30, 0)
    # 生成订单数据
    for oid in range(1, NUM_ORDERS + 1):
        customer_id = random.randint(1, NUM_CUSTOMERS)
        shipping_address_id = ((oid - 1) % NUM_ADDRESSES) + 1
        billing_address_id = ((oid * 3) % NUM_ADDRESSES) + 1
        status = statuses[oid % len(statuses)]
        currency = random.choice(currencies)
        order_date = base_date + timedelta(hours=oid)  # 每小时一个订单
        rows.append(
            [
                oid,
                customer_id,
                order_date.isoformat(),
                status,
                0.0,  # 总金额占位，后续更新
                currency,
                shipping_address_id,
                billing_address_id,
            ]
        )
    return rows


def generate_order_items(
    products: list[list[object]],
) -> tuple[list[list[object]], dict[int, float]]:
    """生成订单明细

    包含字段：
        - order_item_id: 明细ID
        - order_id: 订单ID（外键）
        - product_id: 商品ID（外键）
        - quantity: 数量
        - unit_price: 单价
        - discount_percent: 折扣

    Args:
        products: 商品数据

    Returns:
        (订单明细数据行列表, 订单总金额字典)
    """
    rows = []
    # 用于累计每个订单的总金额
    order_totals: dict[int, float] = {oid: 0.0 for oid in range(1, NUM_ORDERS + 1)}
    # 生成订单明细
    for item_id in range(1, NUM_ORDER_ITEMS + 1):
        order_id = ((item_id - 1) % NUM_ORDERS) + 1  # 循环分配到订单
        product_row = random.choice(products)
        product_id = product_row[0]
        price = float(product_row[3])
        quantity = random.randint(1, 5)
        discount = 0.0
        # 部分订单有折扣
        if item_id % 15 == 0:
            discount = 5.0
        elif item_id % 40 == 0:
            discount = 10.0
        line_total = max(price * quantity - discount, 0)  # 行总价
        order_totals[order_id] += line_total  # 累计到订单
        rows.append(
            [
                item_id,
                order_id,
                product_id,
                quantity,
                round(price, 2),
                round(discount, 2),
            ]
        )
    return rows, order_totals


def update_order_totals(
    orders: list[list[object]], order_totals: dict[int, float]
) -> None:
    """更新订单总金额

    根据订单明细计算的总金额，更新订单表中的 total_amount 字段。
    同时加上运费（5-15元不等）。

    Args:
        orders: 订单数据
        order_totals: 订单总金额字典
    """
    for row in orders:
        order_id = row[0]
        subtotal = round(order_totals.get(order_id, 0.0), 2)
        shipping_cost = round(5 + (order_id % 4) * 2.5, 2)  # 运费: 5/7.5/10/12.5
        row[4] = round(subtotal + shipping_cost, 2)  # 更新总金额


def generate_payments(orders: list[list[object]]) -> list[list[object]]:
    """生成支付记录

    包含字段：
        - payment_id: 支付ID
        - order_id: 订单ID（外键）
        - method: 支付方式
        - status: 支付状态
        - amount: 支付金额
        - transaction_reference: 交易流水号
        - paid_at: 支付时间

    Args:
        orders: 订单数据

    Returns:
        支付记录数据行列表
    """
    methods = ["CARD", "PAYPAL", "BANK_TRANSFER", "APPLE_PAY", "WECHAT_PAY"]
    statuses = ["COMPLETED", "PENDING", "FAILED", "REFUNDED"]
    rows = []
    # 为每个订单生成支付记录
    for pid, order_row in enumerate(orders, start=1):
        order_id = order_row[0]
        amount = order_row[4]
        method = random.choice(methods)
        status = statuses[order_id % len(statuses)]
        # 取消的订单状态改为退款
        if order_row[3] == "CANCELLED":
            status = "REFUNDED"
        transaction_ref = f"TX-{order_id:06d}-{pid:04d}"
        paid_at = datetime.fromisoformat(order_row[2]) + timedelta(minutes=30)  # 下单30分钟后支付
        rows.append(
            [
                pid,
                order_id,
                method,
                status,
                amount,
                transaction_ref,
                paid_at.isoformat(),
            ]
        )
    return rows


def generate_shipments(orders: list[list[object]]) -> list[list[object]]:
    """生成物流信息

    包含字段：
        - shipment_id: 物流ID
        - order_id: 订单ID（外键）
        - carrier: 物流公司
        - tracking_number: 物流单号
        - status: 物流状态
        - shipped_at: 发货时间
        - delivered_at: 送达时间
        - destination_country: 目的地国家

    Args:
        orders: 订单数据

    Returns:
        物流信息数据行列表
    """
    carriers = ["FedEx", "UPS", "DHL", "顺丰速运", "Correos"]
    statuses = ["PENDING", "IN_TRANSIT", "DELIVERED", "RETURNED"]
    rows = []
    # 为每个订单生成物流记录
    for sid, order_row in enumerate(orders, start=1):
        order_id = order_row[0]
        status = statuses[sid % len(statuses)]
        shipped_at = datetime.fromisoformat(order_row[2]) + timedelta(days=1)  # 下单1天后发货
        delivered_at = ""
        # 已送达的订单记录送达时间
        if status == "DELIVERED":
            delivered_at = (shipped_at + timedelta(days=3)).isoformat()  # 发货3天后送达
        tracking = f"TRK{sid:08d}"
        destination_country = random.choice(["USA", "中国", "España", "Canada", "日本"])
        rows.append(
            [
                sid,
                order_id,
                random.choice(carriers),
                tracking,
                status,
                shipped_at.isoformat(),
                delivered_at,
                destination_country,
            ]
        )
    return rows


def generate_reviews() -> list[list[object]]:
    """生成商品评价

    包含字段：
        - review_id: 评价ID
        - product_id: 商品ID（外键）
        - customer_id: 客户ID（外键）
        - rating: 评分（1-5星）
        - title_en/zh/es: 标题（多语言）
        - body_en/zh/es: 内容（多语言）
        - created_at: 创建时间

    Returns:
        评价数据行列表
    """
    titles_en = [
        "Great quality",
        "Not bad",
        "Value for money",
        "Disappointing",
        "Exceeded expectations",
    ]
    titles_zh = ["品质很好", "还可以", "性价比高", "有点失望", "超出预期"]
    titles_es = [
        "Gran calidad",
        "No está mal",
        "Buena relación calidad-precio",
        "Decepcionante",
        "Superó expectativas",
    ]
    bodies_en = [
        "Product works perfectly for daily use.",
        "Packaging arrived with minor dents but overall fine.",
        "Color matches the photos and multi-language manual included.",
        "Battery life shorter than advertised.",
        "Excellent craftsmanship and fast shipping.",
    ]
    bodies_zh = [
        "产品日常使用效果很好。",
        "包装有点磕碰但总体不错。",
        "颜色与图片一致，附带多语言说明书。",
        "续航时间没有宣传的长。",
        "做工精致，发货速度快。",
    ]
    bodies_es = [
        "Funciona perfecto para el uso diario.",
        "El embalaje llegó con pequeños golpes pero aceptable.",
        "El color coincide con las fotos y trae manual multilingüe.",
        "La batería dura menos de lo anunciado.",
        "Excelente calidad y envío rápido.",
    ]
    rows = []
    base_date = datetime(2023, 8, 1, 12, 0, 0)
    # 生成评价数据
    for rid in range(1, NUM_REVIEWS + 1):
        product_id = ((rid - 1) % NUM_PRODUCTS) + 1  # 循环分配到商品
        customer_id = ((rid * 7) % NUM_CUSTOMERS) + 1  # 分散到不同客户
        rating = random.randint(1, 5)
        idx = rid % len(titles_en)
        created_at = base_date + timedelta(minutes=rid)  # 每分钟一条评价
        rows.append(
            [
                rid,
                product_id,
                customer_id,
                rating,
                titles_en[idx],
                titles_zh[idx],
                titles_es[idx],
                bodies_en[idx],
                bodies_zh[idx],
                bodies_es[idx],
                created_at.isoformat(),
            ]
        )
    return rows


def generate_support_tickets() -> list[list[object]]:
    """生成客服工单

    包含字段：
        - ticket_id: 工单ID
        - customer_id: 客户ID（外键）
        - subject_en/zh/es: 主题（多语言）
        - channel: 渠道（邮件/电话/在线客服等）
        - priority: 优先级
        - status: 工单状态
        - created_at: 创建时间
        - resolved_at: 解决时间

    Returns:
        工单数据行列表
    """
    subjects = {
        "en": [
            "Payment issue",
            "Shipping delay",
            "Account access",
            "Warranty request",
            "Feedback",
        ],
        "zh": ["支付问题", "发货延迟", "账户登录", "保修申请", "意见反馈"],
        "es": [
            "Problema de pago",
            "Retraso de envío",
            "Acceso a la cuenta",
            "Solicitud de garantía",
            "Comentarios",
        ],
    }
    channels = ["email", "phone", "chat", "wechat", "whatsapp"]
    priorities = ["LOW", "MEDIUM", "HIGH", "URGENT"]
    statuses = ["OPEN", "IN_PROGRESS", "WAITING_CUSTOMER", "RESOLVED", "CLOSED"]
    rows = []
    base_date = datetime(2023, 6, 1, 10, 0, 0)
    # 生成工单数据
    for tid in range(1, NUM_TICKETS + 1):
        customer_id = ((tid * 11) % NUM_CUSTOMERS) + 1  # 分散到不同客户
        idx = tid % len(subjects["en"])
        created_at = base_date + timedelta(hours=tid)  # 每小时一个工单
        resolved_at = ""
        status = statuses[tid % len(statuses)]
        # 已解决和已关闭的工单记录解决时间
        if status in {"RESOLVED", "CLOSED"}:
            resolved_at = (created_at + timedelta(days=2)).isoformat()  # 2天后解决
        rows.append(
            [
                tid,
                customer_id,
                subjects["en"][idx],
                subjects["zh"][idx],
                subjects["es"][idx],
                random.choice(channels),
                random.choice(priorities),
                status,
                created_at.isoformat(),
                resolved_at,
            ]
        )
    return rows


def main() -> None:
    """主函数：生成所有表的测试数据"""
    print("=" * 60)
    print("Table 测试数据生成")
    print("=" * 60)
    print(f"输出目录: {OUTPUT_DIR}")
    print()
    print("数据量配置:")
    print(f"  客户数量: {NUM_CUSTOMERS:,}")
    print(f"  地址数量: {NUM_ADDRESSES:,}")
    print(f"  分类数量: {NUM_CATEGORIES:,}")
    print(f"  商品数量: {NUM_PRODUCTS:,}")
    print(f"  订单数量: {NUM_ORDERS:,}")
    print(f"  订单明细: {NUM_ORDER_ITEMS:,}")
    print(f"  支付记录: {NUM_PAYMENTS:,}")
    print(f"  物流记录: {NUM_SHIPMENTS:,}")
    print(f"  评价数量: {NUM_REVIEWS:,}")
    print(f"  工单数量: {NUM_TICKETS:,}")
    print()

    print("开始生成数据...")
    print()

    print("[1/11] 客户信息 (customer)")
    customers = generate_customers()
    print(f"  ✓ 生成 {len(customers):,} 条客户记录")
    print()

    print("[2/11] 客户地址 (customer_address)")
    addresses = generate_addresses()
    print(f"  ✓ 生成 {len(addresses):,} 条地址记录")
    print()

    print("[3/11] 商品分类 (category)")
    categories = generate_categories()
    print(f"  ✓ 生成 {len(categories):,} 条分类记录（含父子层级）")
    print()

    print("[4/11] 商品信息 (products)")
    products, _prices = generate_products(categories)
    print(f"  ✓ 生成 {len(products):,} 条商品记录")
    print()

    print("[5/11] 商品翻译 (product_translation)")
    product_translations = generate_product_translations(products)
    print(f"  ✓ 生成 {len(product_translations):,} 条翻译记录（{len(products)} 商品 × 3 语言）")
    print()

    print("[6/11] 订单 (order)")
    orders = generate_orders(customers, addresses)
    print(f"  ✓ 生成 {len(orders):,} 条订单记录")
    print()

    print("[7/11] 订单明细 (order_items)")
    order_items, order_totals = generate_order_items(products)
    print(f"  ✓ 生成 {len(order_items):,} 条订单明细记录")
    print()

    print("[8/11] 更新订单总金额")
    update_order_totals(orders, order_totals)
    print(f"  ✓ 更新 {len(orders):,} 条订单的总金额（含运费）")
    print()

    print("[9/11] 支付记录 (payment)")
    payments = generate_payments(orders)
    print(f"  ✓ 生成 {len(payments):,} 条支付记录")
    print()

    print("[10/11] 物流信息 (shipment)")
    shipments = generate_shipments(orders)
    print(f"  ✓ 生成 {len(shipments):,} 条物流记录")
    print()

    print("[11/11] 评价与工单")
    reviews = generate_reviews()
    print(f"  ✓ 生成 {len(reviews):,} 条商品评价")
    tickets = generate_support_tickets()
    print(f"  ✓ 生成 {len(tickets):,} 条客服工单")
    print()

    print("写入 CSV 文件...")
    print()
    write_csv(
        "customer.csv",
        [
            "customer_id",
            "full_name",
            "email",
            "phone",
            "locale",
            "status",
            "created_at",
            "loyalty_points",
        ],
        customers,
    )
    write_csv(
        "customer_address.csv",
        [
            "address_id",
            "customer_id",
            "address_type",
            "line1",
            "line2",
            "city",
            "region",
            "postal_code",
            "country",
            "latitude",
            "longitude",
        ],
        addresses,
    )
    write_csv(
        "category.csv",
        [
            "category_id",
            "parent_category_id",
            "slug",
            "display_name_en",
            "display_name_zh",
            "display_name_es",
            "description",
            "created_at",
        ],
        categories,
    )
    write_csv(
        "products.csv",
        [
            "product_id",
            "category_id",
            "sku",
            "price",
            "cost",
            "currency",
            "status",
            "created_at",
        ],
        products,
    )
    write_csv(
        "product_translation.csv",
        [
            "product_id",
            "locale",
            "name",
            "description",
        ],
        product_translations,
    )
    write_csv(
        "order.csv",
        [
            "order_id",
            "customer_id",
            "order_date",
            "status",
            "total_amount",
            "currency",
            "shipping_address_id",
            "billing_address_id",
        ],
        orders,
    )
    write_csv(
        "order_items.csv",
        [
            "order_item_id",
            "order_id",
            "product_id",
            "quantity",
            "unit_price",
            "discount_percent",
        ],
        order_items,
    )
    write_csv(
        "payment.csv",
        [
            "payment_id",
            "order_id",
            "method",
            "status",
            "amount",
            "transaction_reference",
            "paid_at",
        ],
        payments,
    )
    write_csv(
        "shipment.csv",
        [
            "shipment_id",
            "order_id",
            "carrier",
            "tracking_number",
            "status",
            "shipped_at",
            "delivered_at",
            "destination_country",
        ],
        shipments,
    )
    write_csv(
        "product_review.csv",
        [
            "review_id",
            "product_id",
            "customer_id",
            "rating",
            "title_en",
            "title_zh",
            "title_es",
            "body_en",
            "body_zh",
            "body_es",
            "created_at",
        ],
        reviews,
    )
    write_csv(
        "support_ticket.csv",
        [
            "ticket_id",
            "customer_id",
            "subject_en",
            "subject_zh",
            "subject_es",
            "channel",
            "priority",
            "status",
            "created_at",
            "resolved_at",
        ],
        tickets,
    )

    print()
    print("=" * 60)
    print("✅ 生成完成!")
    print("=" * 60)
    print("数据统计:")
    print(f"  客户信息:           {len(customers):>6,} 行")
    print(f"  客户地址:           {len(addresses):>6,} 行")
    print(f"  商品分类:           {len(categories):>6,} 行")
    print(f"  商品信息:           {len(products):>6,} 行")
    print(f"  商品翻译:           {len(product_translations):>6,} 行")
    print(f"  订单:               {len(orders):>6,} 行")
    print(f"  订单明细:           {len(order_items):>6,} 行")
    print(f"  支付记录:           {len(payments):>6,} 行")
    print(f"  物流信息:           {len(shipments):>6,} 行")
    print(f"  商品评价:           {len(reviews):>6,} 行")
    print(f"  客服工单:           {len(tickets):>6,} 行")
    print(f"  {'─' * 30}")
    total_rows = (
        len(customers)
        + len(addresses)
        + len(categories)
        + len(products)
        + len(product_translations)
        + len(orders)
        + len(order_items)
        + len(payments)
        + len(shipments)
        + len(reviews)
        + len(tickets)
    )
    print(f"  总计:               {total_rows:>6,} 行")
    print(f"  总文件数:           11 个 CSV 文件")
    print()


if __name__ == "__main__":
    main()
