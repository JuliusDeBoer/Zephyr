-- Your SQL goes here
CREATE TABLE "users"(
	"id" UUID NOT NULL PRIMARY KEY,
	"email" TEXT NOT NULL UNIQUE,
	"password" CHAR(97) NOT NULL,
	"first_name" TEXT NOT NULL,
	"affix" TEXT,
	"last_name" TEXT NOT NULL
);
