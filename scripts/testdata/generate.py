#!/usr/bin/env python3
from __future__ import annotations

import csv
import random
from datetime import datetime, timedelta
from pathlib import Path


random.seed(42)

NUM_CUSTOMERS = 1000
NUM_ADDRESSES = 1000
NUM_CATEGORIES = 1000
NUM_PRODUCTS = 1000
NUM_ORDERS = 1000
NUM_ORDER_ITEMS = 3000
NUM_PAYMENTS = 1000
NUM_SHIPMENTS = 1000
NUM_REVIEWS = 1000
NUM_TICKETS = 1000

OUTPUT_DIR = Path(__file__).parent / "output"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)


def write_csv(filename: str, header: list[str], rows: list[list[object]]) -> None:
    with (OUTPUT_DIR / filename).open("w", newline="", encoding="utf-8") as fp:
        writer = csv.writer(fp)
        writer.writerow(header)
        writer.writerows(rows)


def generate_customers() -> list[list[object]]:
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

    for cid in range(1, NUM_CUSTOMERS + 1):
        locale = random.choice(locales)
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
    for aid in range(1, NUM_ADDRESSES + 1):
        customer_id = ((aid - 1) % NUM_CUSTOMERS) + 1
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
    for cid in range(1, NUM_CATEGORIES + 1):
        slug_base = random.choice(base_names)
        slug = f"{slug_base[0]}-{cid}"
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
    statuses = ["ACTIVE", "DISCONTINUED"]
    currencies = ["USD", "CNY", "EUR", "JPY"]
    rows = []
    prices = []
    base_date = datetime(2022, 6, 1, 10, 0, 0)
    category_ids = [row[0] for row in categories]

    for pid in range(1, NUM_PRODUCTS + 1):
        category_id = random.choice(category_ids)
        sku = f"SKU{pid:05d}"
        price = round(10 + (pid * 2.37) % 500, 2)
        cost = round(price * random.uniform(0.4, 0.8), 2)
        currency = random.choice(currencies)
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
    statuses = ["PENDING", "PAID", "SHIPPED", "COMPLETED", "CANCELLED"]
    currencies = ["USD", "CNY", "EUR", "JPY"]
    rows = []
    base_date = datetime(2023, 7, 1, 9, 30, 0)
    for oid in range(1, NUM_ORDERS + 1):
        customer_id = random.randint(1, NUM_CUSTOMERS)
        shipping_address_id = ((oid - 1) % NUM_ADDRESSES) + 1
        billing_address_id = ((oid * 3) % NUM_ADDRESSES) + 1
        status = statuses[oid % len(statuses)]
        currency = random.choice(currencies)
        order_date = base_date + timedelta(hours=oid)
        rows.append(
            [
                oid,
                customer_id,
                order_date.isoformat(),
                status,
                0.0,  # placeholder total, will update later
                currency,
                shipping_address_id,
                billing_address_id,
            ]
        )
    return rows


def generate_order_items(
    products: list[list[object]],
) -> tuple[list[list[object]], dict[int, float]]:
    rows = []
    order_totals: dict[int, float] = {oid: 0.0 for oid in range(1, NUM_ORDERS + 1)}
    for item_id in range(1, NUM_ORDER_ITEMS + 1):
        order_id = ((item_id - 1) % NUM_ORDERS) + 1
        product_row = random.choice(products)
        product_id = product_row[0]
        price = float(product_row[3])
        quantity = random.randint(1, 5)
        discount = 0.0
        if item_id % 15 == 0:
            discount = 5.0
        elif item_id % 40 == 0:
            discount = 10.0
        line_total = max(price * quantity - discount, 0)
        order_totals[order_id] += line_total
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
    for row in orders:
        order_id = row[0]
        subtotal = round(order_totals.get(order_id, 0.0), 2)
        shipping_cost = round(5 + (order_id % 4) * 2.5, 2)
        row[4] = round(subtotal + shipping_cost, 2)


def generate_payments(orders: list[list[object]]) -> list[list[object]]:
    methods = ["CARD", "PAYPAL", "BANK_TRANSFER", "APPLE_PAY", "WECHAT_PAY"]
    statuses = ["COMPLETED", "PENDING", "FAILED", "REFUNDED"]
    rows = []
    for pid, order_row in enumerate(orders, start=1):
        order_id = order_row[0]
        amount = order_row[4]
        method = random.choice(methods)
        status = statuses[order_id % len(statuses)]
        if order_row[3] == "CANCELLED":
            status = "REFUNDED"
        transaction_ref = f"TX-{order_id:06d}-{pid:04d}"
        paid_at = datetime.fromisoformat(order_row[2]) + timedelta(minutes=30)
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
    carriers = ["FedEx", "UPS", "DHL", "顺丰速运", "Correos"]
    statuses = ["PENDING", "IN_TRANSIT", "DELIVERED", "RETURNED"]
    rows = []
    for sid, order_row in enumerate(orders, start=1):
        order_id = order_row[0]
        status = statuses[sid % len(statuses)]
        shipped_at = datetime.fromisoformat(order_row[2]) + timedelta(days=1)
        delivered_at = ""
        if status == "DELIVERED":
            delivered_at = (shipped_at + timedelta(days=3)).isoformat()
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
    for rid in range(1, NUM_REVIEWS + 1):
        product_id = ((rid - 1) % NUM_PRODUCTS) + 1
        customer_id = ((rid * 7) % NUM_CUSTOMERS) + 1
        rating = random.randint(1, 5)
        idx = rid % len(titles_en)
        created_at = base_date + timedelta(minutes=rid)
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
    for tid in range(1, NUM_TICKETS + 1):
        customer_id = ((tid * 11) % NUM_CUSTOMERS) + 1
        idx = tid % len(subjects["en"])
        created_at = base_date + timedelta(hours=tid)
        resolved_at = ""
        status = statuses[tid % len(statuses)]
        if status in {"RESOLVED", "CLOSED"}:
            resolved_at = (created_at + timedelta(days=2)).isoformat()
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
    customers = generate_customers()
    addresses = generate_addresses()
    categories = generate_categories()
    products, _prices = generate_products(categories)
    product_translations = generate_product_translations(products)
    orders = generate_orders(customers, addresses)
    order_items, order_totals = generate_order_items(products)
    update_order_totals(orders, order_totals)
    payments = generate_payments(orders)
    shipments = generate_shipments(orders)
    reviews = generate_reviews()
    tickets = generate_support_tickets()

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


if __name__ == "__main__":
    main()
