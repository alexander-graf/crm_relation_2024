use crate::config::DbConfig;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;



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
    pub customer_id: i32,
}

#[derive(Debug, Clone)]
pub struct ContactHistory {
    pub history_id: i32,
    pub customer_id: i32,
    pub contact_type: String,
    pub contact_date: DateTime<Utc>,
    pub contact_duration: Option<i32>,
    pub contact_method: Option<String>,
    pub contact_outcome: String, // Ändern Sie dies von Option<String> zu String
    pub notes: String,
    pub follow_up_date: Option<NaiveDate>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ContactHistory {
    fn default() -> Self {
        ContactHistory {
            history_id: 0, // Oder ein anderer geeigneter Standardwert
            customer_id: 0,
            contact_type: String::new(),
            contact_date: Utc::now(),
            contact_duration: None,
            contact_method: None,
            contact_outcome: String::new(),
            notes: String::new(),
            follow_up_date: None,
            created_by: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
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

        let grant_query = format!(
            "GRANT ALL PRIVILEGES ON DATABASE {} TO {}",
            config.database, config.username
        );
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

pub async fn create_database_structure(
    config: &DbConfig,
) -> Result<(), Box<dyn std::error::Error>> {
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
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Contact History table
CREATE TABLE IF NOT EXISTS contact_history (
    history_id SERIAL PRIMARY KEY,
    customer_id INTEGER NOT NULL,
    contact_type VARCHAR(50) NOT NULL,
    contact_date TIMESTAMP WITH TIME ZONE NOT NULL,
    contact_duration INTEGER,
    contact_method VARCHAR(50),
    contact_outcome VARCHAR(100) NOT NULL,
    notes TEXT,
    follow_up_date DATE,
    created_by VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id)
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
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
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

pub async fn add_customer(
    config: &DbConfig,
    customer: &Customer,
) -> Result<(), Box<dyn std::error::Error>> {
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

    client
        .execute(
            statement,
            &[
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
            ],
        )
        .await?;

    println!("Customer added successfully");
    Ok(())
}

pub async fn get_customers(config: &DbConfig) -> Result<Vec<Customer>, Box<dyn std::error::Error>> {
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

    let rows = client
        .query("SELECT * FROM customers ORDER BY company_name", &[])
        .await?;

    let customers: Vec<Customer> = rows
        .iter()
        .map(|row| Customer {
            company_name: row.get("company_name"),
            contact_name: row.get("contact_name"),
            contact_position: row.get("contact_position"),
            address: row.get("address"),
            city: row.get("city"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            phone: row.get("phone"),
            email: row.get("email"),
            website: row.get("website"),
            customer_id: row.get("customer_id"),
        })
        .collect();

    Ok(customers)
}

pub async fn get_contact_history(
    config: &DbConfig,
    customer_id: i32,
) -> Result<Vec<ContactHistory>, Box<dyn std::error::Error>> {
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

    let rows = client
        .query(
            "SELECT * FROM contact_history WHERE customer_id = $1 ORDER BY contact_date DESC",
            &[&customer_id],
        )
        .await?;

    let history: Vec<ContactHistory> = rows
        .iter()
        .map(|row| {
            let contact_date: DateTime<Utc> = row.get("contact_date");
            let follow_up_date: Option<NaiveDate> = row.get("follow_up_date");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");

            ContactHistory {
                history_id: row.get("history_id"),
                customer_id: row.get("customer_id"),
                contact_type: row.get("contact_type"),
                contact_date,
                contact_duration: row.get("contact_duration"),
                contact_method: row.get("contact_method"),
                contact_outcome: row.get("contact_outcome"),
                notes: row.get("notes"),
                follow_up_date,
                created_by: row.get("created_by"),
                created_at,
                updated_at,
            }
        })
        .collect();

    Ok(history)
}

pub async fn get_customer_with_history(
    config: &DbConfig,
    customer_id: i32,
) -> Result<(Customer, Vec<ContactHistory>), Box<dyn std::error::Error>> {
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

    println!("Fetching customer data for customer_id: {}", customer_id);
    let customer_row = client
        .query_one(
            "SELECT * FROM customers WHERE customer_id = $1",
            &[&customer_id],
        )
        .await?;

    let customer = Customer {
        customer_id: customer_row.get("customer_id"),
        company_name: customer_row.get("company_name"),
        contact_name: customer_row.get("contact_name"),
        email: customer_row.get("email"),
        phone: customer_row.get("phone"),
        address: customer_row.get("address"),
        city: customer_row.get("city"),
        postal_code: customer_row.get("postal_code"),
        country: customer_row.get("country"),
        website: customer_row.get("website"),
        contact_position: customer_row.get("contact_position"),
    };

    println!("Fetched customer: {:?}", customer);

    println!("Fetching contact history for customer_id: {}", customer_id);
    let history_rows = client
        .query(
            "SELECT * FROM contact_history WHERE customer_id = $1 ORDER BY contact_date DESC",
            &[&customer_id],
        )
        .await?;

    let history: Vec<ContactHistory> = history_rows
        .into_iter()
        .map(|row| {
            let contact_date: DateTime<Utc> = row.get("contact_date");
            let follow_up_date: Option<NaiveDate> = row.get("follow_up_date");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");

            ContactHistory {
                history_id: row.get("history_id"),
                customer_id: row.get("customer_id"),
                contact_type: row.get("contact_type"),
                contact_date,
                contact_duration: row.get("contact_duration"),
                contact_method: row.get("contact_method"),
                contact_outcome: row.get("contact_outcome"),
                notes: row.get("notes"),
                follow_up_date,
                created_by: row.get("created_by"),
                created_at,
                updated_at,
            }
        })
        .collect();

    println!("Fetched {} contact history entries", history.len());

    Ok((customer, history))
}


pub async fn add_contact_history(config: &DbConfig, history: &ContactHistory) -> Result<(), Box<dyn std::error::Error>> {
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
        INSERT INTO contact_history (customer_id, contact_type, contact_date, contact_duration, contact_method, contact_outcome, notes, follow_up_date, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    ";

    client.execute(statement, &[
        &history.customer_id,
        &history.contact_type,
        &history.contact_date,
        &history.contact_duration,
        &history.contact_method,
        &history.contact_outcome,
        &history.notes,
        &history.follow_up_date,
        &history.created_by,
        &history.created_at,
        &history.updated_at,
    ]).await?;

    Ok(())
}
