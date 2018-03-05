use schema::client;
use std::net::IpAddr;

#[derive(Queryable, Serialize, Deserialize, Debug, Insertable, Clone, AsChangeset)]
#[table_name = "client"]
pub struct Client {
    pub mesh_ip: String,
    pub wg_pubkey: String,
    pub wg_port: String,
    pub luci_pass: String,
    pub internal_ip: String,
}