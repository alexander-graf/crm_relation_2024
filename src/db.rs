use serde::{Serialize, Deserialize};
use tokio_postgres::{NoTls, Error as PgError};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct DbConfig {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Customer {
    pub company_name: String,
    pub contact_name: String,
    pub contact_position: String,
    pub address: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
    pub phone: String,
    pub email: String,
    pub website: String,
}

pub async fn create_database(config: &DbConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to create database: {}", config.database);
    let conn_string = format!(
        "host={} port={} user=postgres dbname=postgres",
        config.host, config.port
    );

    println!("Connecting to PostgreSQL server as postgres user...");
    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query = "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)";
    println!("Executing query: {}", query);
    let db_exists: bool = client.query_one(query, &[&config.database]).await?.get(0);

    if !db_exists {
        println!("Database does not exist. Creating...");
        let create_db_query = format!("CREATE DATABASE {}", config.database);
        println!("Executing query: {}", create_db_query);
        client.execute(&create_db_query, &[]).await?;
        println!("Database created successfully");

        let grant_query = format!("GRANT ALL PRIVILEGES ON DATABASE {} TO {}", config.database, config.username);
        println!("Executing query: {}", grant_query);
        client.execute(&grant_query, &[]).await?;
        println!("Privileges granted to user {}", config.username);
    } else {
        println!("Database already exists");
    }

    Ok(())
}

use once_cell::sync::Lazy;

use std::sync::Mutex;

static DB_CONFIG: Lazy<Mutex<Option<DbConfig>>> = Lazy::new(|| Mutex::new(None));

pub fn set_config(config: Option<DbConfig>) {
    let mut db_config = DB_CONFIG.lock().unwrap();
    *db_config = config.clone();
    println!("Configuration set: {:?}", config);
}
pub fn get_config() -> Option<DbConfig> {
    let config = DB_CONFIG.lock().unwrap().clone();
    println!("Retrieved configuration: {:?}", config);
    config
}

pub async fn create_database_structure(config: &DbConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating database structure for: {}", config.database);
    let conn_string = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host, config.port, config.username, config.password, config.database
    );

    println!("Connecting to database...");
    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("Connected successfully. Creating tables...");
    
    let create_tables_query = "
        -- Customers table
        CREATE TABLE IF NOT EXISTS customers (
            customer_id SERIAL PRIMARY KEY,
            company_name VARCHAR(100) NOT NULL,
            contact_name VARCHAR(100),
            contact_position VARCHAR(50),
            address TEXT,
            city VARCHAR(50),
            postal_code VARCHAR(20),
            country VARCHAR(50),
            phone VARCHAR(20),
            email VARCHAR(100),
            website VARCHAR(100),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Products table
        CREATE TABLE IF NOT EXISTS products (
            product_id SERIAL PRIMARY KEY,
            product_name VARCHAR(100) NOT NULL,
            description TEXT,
            unit_price DECIMAL(10, 2) NOT NULL,
            stock_quantity INTEGER NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Invoices table
        CREATE TABLE IF NOT EXISTS invoices (
            invoice_id SERIAL PRIMARY KEY,
            customer_id INTEGER REFERENCES customers(customer_id),
            invoice_number VARCHAR(50) UNIQUE NOT NULL,
            invoice_date DATE NOT NULL,
            due_date DATE NOT NULL,
            total_amount DECIMAL(10, 2) NOT NULL,
            status VARCHAR(20) NOT NULL,
            payment_method VARCHAR(50),
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

               -- Invoice Items table
        CREATE TABLE IF NOT EXISTS invoice_items (
            item_id SERIAL PRIMARY KEY,
            invoice_id INTEGER REFERENCES invoices(invoice_id),
            product_id INTEGER REFERENCES products(product_id),
            quantity INTEGER NOT NULL,
            unit_price DECIMAL(10, 2) NOT NULL,
            total_price DECIMAL(10, 2) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Payments table
        CREATE TABLE IF NOT EXISTS payments (
            payment_id SERIAL PRIMARY KEY,
            invoice_id INTEGER REFERENCES invoices(invoice_id),
            payment_date DATE NOT NULL,
            amount DECIMAL(10, 2) NOT NULL,
            payment_method VARCHAR(50) NOT NULL,
            transaction_id VARCHAR(100),
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Contacts table
        CREATE TABLE IF NOT EXISTS contacts (
            contact_id SERIAL PRIMARY KEY,
            customer_id INTEGER REFERENCES customers(customer_id),
            first_name VARCHAR(50) NOT NULL,
            last_name VARCHAR(50) NOT NULL,
            email VARCHAR(100),
            phone VARCHAR(20),
            position VARCHAR(50),
            is_primary BOOLEAN DEFAULT false,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Create indexes for better performance
        CREATE INDEX IF NOT EXISTS idx_customers_company_name ON customers(company_name);
        CREATE INDEX IF NOT EXISTS idx_products_product_name ON products(product_name);
        CREATE INDEX IF NOT EXISTS idx_invoices_customer_id ON invoices(customer_id);
        CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice_id ON invoice_items(invoice_id);
        CREATE INDEX IF NOT EXISTS idx_invoice_items_product_id ON invoice_items(product_id);
        CREATE INDEX IF NOT EXISTS idx_payments_invoice_id ON payments(invoice_id);
        CREATE INDEX IF NOT EXISTS idx_contacts_customer_id ON contacts(customer_id);
    ";

    println!("Executing query to create tables and set relationships...");
    client.batch_execute(create_tables_query).await?;

    println!("Database structure created successfully");
    Ok(())
}

pub async fn add_customer(config: &DbConfig, customer: &Customer) -> Result<(), Box<dyn std::error::Error>> {
    let conn_string = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host, config.port, config.username, config.password, config.database
    );

    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let statement = "
        INSERT INTO customers (company_name, contact_name, contact_position, address, city, postal_code, country, phone, email, website)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ";

    client.execute(statement, &[
        &customer.company_name,
        &customer.contact_name,
        &customer.contact_position,
        &customer.address,
        &customer.city,
        &customer.postal_code,
        &customer.country,
        &customer.phone,
        &customer.email,
        &customer.website,
    ]).await?;

    println!("Customer added successfully");
    Ok(())
}


