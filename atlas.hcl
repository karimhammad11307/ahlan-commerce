variable "db_url" {
  type    = string
  default = "postgresql://ahlan:ahlan_dev@localhost:5432/ahlan_commerce?sslmode=disable"
}

env "local" {
  src = "file://db/schema/products.sql"
  dev = "postgresql://ahlan:ahlan_dev@localhost:5432/ahlan_commerce?search_path=atlas_dev&sslmode=disable"
  
  migration {
    dir = "file://db/migrations"
  }
  
  url = var.db_url
}
