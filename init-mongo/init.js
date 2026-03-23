db = db.getSiblingDB("mydb");
db.createUser({
    user: "content",
    pwd: "dev_pass",
    roles: [
        { role: "readWrite", db: "content" }
    ]
});

db.createCollection("content");