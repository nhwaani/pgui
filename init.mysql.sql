-- MySQL 8.4-compatible seed schema for testing pgui's MySQL backend.
-- Mirrors the structure of init.sql (the Postgres seed) where it makes
-- sense, but uses MySQL's flavor of types, indexes, and check
-- constraints. No PG-specific extensions (arrays, ranges, geometric
-- types, full-text vectors, etc.).
--
-- This script runs automatically on the first start of the mysql
-- container (the container only invokes /docker-entrypoint-initdb.d
-- when MYSQL_DATABASE has just been created). To rebuild from
-- scratch:
--   docker compose down -v && docker compose up -d mysql

USE test;

-- ============================================================================
-- Users
-- ============================================================================
CREATE TABLE users (
  id           INT AUTO_INCREMENT PRIMARY KEY,
  name         VARCHAR(50),
  email        VARCHAR(100) UNIQUE,
  created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  is_active    BOOLEAN NOT NULL DEFAULT TRUE
);

INSERT INTO users (name, email) VALUES
  ('Alpha',   'alpha@example.com'),
  ('Beta',    'beta@example.com'),
  ('Gamma',   'gamma@example.com'),
  ('Delta',   'delta@example.com'),
  ('Echo',    'echo@example.com'),
  ('Foxtrot', 'foxtrot@example.com');

-- ============================================================================
-- Companies
-- ============================================================================
CREATE TABLE companies (
  id              INT AUTO_INCREMENT PRIMARY KEY,
  name            VARCHAR(100) NOT NULL,
  industry        VARCHAR(50),
  founded_year    INT,
  headquarters    VARCHAR(100),
  website         VARCHAR(200),
  employee_count  INT,
  annual_revenue  DECIMAL(15,2),
  is_public       BOOLEAN NOT NULL DEFAULT FALSE,
  created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO companies (name, industry, founded_year, headquarters, website, employee_count, annual_revenue, is_public) VALUES
  ('TechCorp Solutions',     'Technology',       2010, 'San Francisco, CA', 'https://techcorp.com',       1250,  89500000.00, TRUE),
  ('Global Manufacturing',   'Manufacturing',    1985, 'Detroit, MI',       'https://globalmfg.com',      5600, 230000000.00, TRUE),
  ('Green Energy Partners',  'Renewable Energy', 2015, 'Austin, TX',        'https://greenenergy.com',     340,  12800000.00, FALSE),
  ('Digital Marketing Hub',  'Marketing',        2018, 'New York, NY',      'https://dmhub.com',            85,   5200000.00, FALSE),
  ('Healthcare Innovations', 'Healthcare',       2005, 'Boston, MA',        'https://healthinnovate.com',  890,  45600000.00, FALSE);

-- ============================================================================
-- Categories (self-referential FK to exercise pgui's FK introspection)
-- ============================================================================
CREATE TABLE categories (
  id          INT AUTO_INCREMENT PRIMARY KEY,
  name        VARCHAR(50) NOT NULL,
  description TEXT,
  parent_id   INT,
  sort_order  INT NOT NULL DEFAULT 0,
  is_active   BOOLEAN NOT NULL DEFAULT TRUE,
  CONSTRAINT fk_categories_parent
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);

INSERT INTO categories (name, description, parent_id, sort_order) VALUES
  ('Electronics',    'Electronic devices and components',     NULL, 1),
  ('Computers',      'Desktop and laptop computers',          1,    1),
  ('Mobile Devices', 'Phones, tablets, and accessories',      1,    2),
  ('Home & Garden',  'Home improvement and gardening',        NULL, 2),
  ('Furniture',      'Indoor and outdoor furniture',          4,    1),
  ('Tools',          'Hand tools and power tools',            4,    2),
  ('Books',          'Physical and digital books',            NULL, 3),
  ('Fiction',        'Novels and short stories',              7,    1),
  ('Non-Fiction',    'Educational and reference books',       7,    2);

-- ============================================================================
-- Products
-- ============================================================================
CREATE TABLE products (
  id               INT AUTO_INCREMENT PRIMARY KEY,
  sku              VARCHAR(20) NOT NULL UNIQUE,
  name             VARCHAR(100) NOT NULL,
  description      TEXT,
  category_id      INT,
  price            DECIMAL(10,2) NOT NULL,
  cost             DECIMAL(10,2),
  stock_quantity   INT NOT NULL DEFAULT 0,
  min_stock_level  INT NOT NULL DEFAULT 5,
  weight_kg        DECIMAL(6,2),
  dimensions_cm    VARCHAR(20),
  manufacturer     VARCHAR(50),
  warranty_months  INT NOT NULL DEFAULT 12,
  is_discontinued  BOOLEAN NOT NULL DEFAULT FALSE,
  created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_products_category
    FOREIGN KEY (category_id) REFERENCES categories(id),
  CONSTRAINT chk_products_price_nonneg CHECK (price >= 0)
);

CREATE INDEX idx_products_sku      ON products(sku);
CREATE INDEX idx_products_category ON products(category_id);

INSERT INTO products (sku, name, description, category_id, price, cost, stock_quantity, min_stock_level, weight_kg, dimensions_cm, manufacturer, warranty_months) VALUES
  ('LAPTOP001', 'UltraBook Pro 15',        'High-performance laptop with 16GB RAM and 512GB SSD', 2, 1299.99,  850.00,  25,  5,  1.80, '35x24x2',   'TechCorp',          24),
  ('PHONE001',  'SmartPhone X',            'Latest smartphone with advanced camera system',      3,  899.99,  600.00, 150, 20,  0.18, '15x7x1',    'MobileTech',        12),
  ('CHAIR001',  'Ergonomic Office Chair',  'Adjustable office chair with lumbar support',         5,  249.99,  125.00,  45, 10, 15.50, '60x60x120', 'ComfortSeating',    36),
  ('DRILL001',  'Cordless Power Drill',    '18V cordless drill with 2 batteries',                 6,   89.99,   45.00,  78, 15,  1.20, '25x8x20',   'PowerTools Pro',    24),
  ('BOOK001',   'The Art of Programming',  'Comprehensive guide to software development',         9,   49.99,   25.00, 200, 25,  0.80, '24x17x3',   'Tech Publishers',    0);

-- ============================================================================
-- Orders + items
-- ============================================================================
CREATE TABLE order_statuses (
  id          INT AUTO_INCREMENT PRIMARY KEY,
  name        VARCHAR(30) NOT NULL UNIQUE,
  description VARCHAR(100),
  sort_order  INT NOT NULL DEFAULT 0
);

INSERT INTO order_statuses (name, description, sort_order) VALUES
  ('pending',    'Order received, awaiting processing', 1),
  ('processing', 'Order is being prepared',             2),
  ('shipped',    'Order has been shipped',              3),
  ('delivered',  'Order has been delivered',            4),
  ('cancelled',  'Order has been cancelled',            5),
  ('returned',   'Order has been returned',             6);

CREATE TABLE orders (
  id               INT AUTO_INCREMENT PRIMARY KEY,
  order_number     VARCHAR(20) NOT NULL UNIQUE,
  user_id          INT,
  company_id       INT,
  status_id        INT NOT NULL DEFAULT 1,
  order_date       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  ship_date        TIMESTAMP NULL,
  total_amount     DECIMAL(12,2) NOT NULL,
  tax_amount       DECIMAL(10,2) NOT NULL DEFAULT 0,
  shipping_amount  DECIMAL(8,2)  NOT NULL DEFAULT 0,
  discount_amount  DECIMAL(10,2) NOT NULL DEFAULT 0,
  shipping_address TEXT,
  billing_address  TEXT,
  notes            TEXT,
  CONSTRAINT fk_orders_user    FOREIGN KEY (user_id)    REFERENCES users(id),
  CONSTRAINT fk_orders_company FOREIGN KEY (company_id) REFERENCES companies(id),
  CONSTRAINT fk_orders_status  FOREIGN KEY (status_id)  REFERENCES order_statuses(id)
);

CREATE INDEX idx_orders_user ON orders(user_id);
CREATE INDEX idx_orders_date ON orders(order_date);

INSERT INTO orders (order_number, user_id, company_id, status_id, order_date, total_amount, tax_amount, shipping_amount, shipping_address, billing_address) VALUES
  ('ORD-2024-001', 1, 1, 3, '2024-01-15 10:30:00', 1549.98, 124.00, 25.99, '123 Main St, Anytown, ST 12345', '123 Main St, Anytown, ST 12345'),
  ('ORD-2024-002', 2, 2, 4, '2024-01-18 14:22:00',  899.99,  72.00, 15.99, '456 Oak Ave, Somewhere, ST 67890', '456 Oak Ave, Somewhere, ST 67890'),
  ('ORD-2024-003', 3, 1, 2, '2024-01-20 09:15:00',  339.97,  27.20, 12.99, '789 Pine Rd, Elsewhere, ST 54321', '789 Pine Rd, Elsewhere, ST 54321'),
  ('ORD-2024-004', 4, 3, 1, '2024-01-22 16:45:00',  139.98,  11.20,  8.99, '321 Elm St, Nowhere, ST 98765',    '321 Elm St, Nowhere, ST 98765');

CREATE TABLE order_items (
  id               INT AUTO_INCREMENT PRIMARY KEY,
  order_id         INT NOT NULL,
  product_id       INT,
  quantity         INT NOT NULL,
  unit_price       DECIMAL(10,2) NOT NULL,
  total_price      DECIMAL(12,2) NOT NULL,
  discount_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
  CONSTRAINT fk_oi_order   FOREIGN KEY (order_id)   REFERENCES orders(id) ON DELETE CASCADE,
  CONSTRAINT fk_oi_product FOREIGN KEY (product_id) REFERENCES products(id)
);

CREATE INDEX idx_oi_order   ON order_items(order_id);
CREATE INDEX idx_oi_product ON order_items(product_id);

INSERT INTO order_items (order_id, product_id, quantity, unit_price, total_price) VALUES
  (1, 1, 1, 1299.99, 1299.99),
  (1, 3, 1,  249.99,  249.99),
  (2, 2, 1,  899.99,  899.99),
  (3, 3, 1,  249.99,  249.99),
  (3, 4, 1,   89.99,   89.99),
  (4, 4, 1,   89.99,   89.99),
  (4, 5, 1,   49.99,   49.99);

-- ============================================================================
-- Data-type variety table
-- A pared-down analogue of advanced_types_test from init.sql, restricted to
-- types that exist in MySQL 8.4. Useful for verifying pgui's MySQL row
-- decoder across numeric / string / date / json types.
-- ============================================================================
CREATE TABLE mysql_types_test (
  id              INT AUTO_INCREMENT PRIMARY KEY,
  tiny_int        TINYINT,
  small_int       SMALLINT,
  regular_int     INT,
  big_int         BIGINT,
  unsigned_big    BIGINT UNSIGNED,
  decimal_val     DECIMAL(10,2),
  float_val       FLOAT,
  double_val      DOUBLE,
  bool_val        BOOLEAN,
  char_fixed      CHAR(10),
  varchar_var     VARCHAR(100),
  text_unlimited  TEXT,
  blob_data       BLOB,
  date_val        DATE,
  time_val        TIME,
  datetime_val    DATETIME,
  timestamp_val   TIMESTAMP NULL,
  enum_val        ENUM('low', 'medium', 'high', 'critical'),
  set_val         SET('reading', 'coding', 'cooking', 'gaming'),
  json_val        JSON,
  nullable_int    INT,
  nullable_text   TEXT,
  created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  notes           TEXT
);

INSERT INTO mysql_types_test
  (tiny_int, small_int, regular_int, big_int, unsigned_big,
   decimal_val, float_val, double_val, bool_val,
   char_fixed, varchar_var, text_unlimited, blob_data,
   date_val, time_val, datetime_val, timestamp_val,
   enum_val, set_val, json_val,
   nullable_int, nullable_text, notes)
VALUES
  (127, 32767, 2147483647, 9223372036854775807, 18446744073709551615,
   12345.67, 3.14, 2.718281828, TRUE,
   'FIXED', 'Variable length string', 'Unlimited text with utf8mb4: üñíçödé 你好 🎉', 0xDEADBEEF,
   '2024-01-15', '14:30:00', '2024-01-15 14:30:00', '2024-01-15 14:30:00',
   'high', 'reading,coding', JSON_OBJECT('name','John','age',30,'hobbies', JSON_ARRAY('reading','coding')),
   42, 'Some text', 'Full row populated'),

  (-128, -32768, -2147483648, -9223372036854775808, 0,
   -99999.99, -3.14, -2.71828, FALSE,
   'ABC',   'Short', 'Multi-line text\nwith line breaks', 0x00010203,
   '1999-12-31', '23:59:59', '1999-12-31 23:59:59', '1999-12-31 23:59:59',
   'critical', 'gaming', JSON_OBJECT('empty', JSON_OBJECT(), 'array', JSON_ARRAY()),
   NULL, NULL, 'Edge cases'),

  (NULL, NULL, NULL, NULL, NULL,
   NULL, NULL, NULL, NULL,
   NULL, NULL, NULL, NULL,
   NULL, NULL, NULL, NULL,
   'low', NULL, NULL,
   NULL, NULL, 'Mostly NULLs');

-- ============================================================================
-- A view, for parity with the Postgres seed.
-- ============================================================================
CREATE OR REPLACE VIEW order_summary AS
  SELECT
    o.id,
    o.order_number,
    u.name  AS customer_name,
    u.email AS customer_email,
    c.name  AS company_name,
    os.name AS status,
    o.order_date,
    o.total_amount,
    COUNT(oi.id) AS item_count
  FROM orders o
  JOIN users u            ON o.user_id   = u.id
  LEFT JOIN companies c   ON o.company_id = c.id
  JOIN order_statuses os  ON o.status_id  = os.id
  LEFT JOIN order_items oi ON o.id        = oi.order_id
  GROUP BY o.id, o.order_number, u.name, u.email, c.name, os.name, o.order_date, o.total_amount
  ORDER BY o.order_date DESC;
