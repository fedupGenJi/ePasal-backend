# ePasal Backend

**2nd Year 2nd Semester Project**  

This is the **backend** for the [ePasal Frontend](https://github.com/fedupGenJi/ePasal-frontend.git) — an e-commerce platform focused on selling laptops.  
The backend is built with **Rust** using **Actix Web**, providing REST APIs for authentication, product management, order handling, and payments.

---

## 📌 Features
- User authentication & authorization  
- Laptop product management  
- Order creation & tracking  
- Payment integration with **Khalti**  
- Email notifications via SMTP  
- PostgreSQL database support  

---

## 🚀 Installation & Setup

### 1️⃣ Clone the Repository
```bash
git clone https://github.com/fedupGenJi/ePasal-backend.git
cd epasal-backend
```

### 2️⃣ Create .env File
```bash
DATABASE_URL= Your postgres database url
(postgres://user:password>@localhost:port/databaseName)

SMTP_EMAIL=
SMTP_PASSWORD=
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587

ADMIN_EMAIL=
ADMIN_PHONE=

BASE_URL=http://frontendUrl
BACKEND_URL=http://backendUrl
KHALTI_SECRET_KEY=
```

### 3️⃣ Install Dependencies
You must have
1. Rust
2. PostgreSQL

### 4️⃣ Start Backend Server
```bash
cargo run
```