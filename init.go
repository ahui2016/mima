package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"

	"ahui2016.github.com/mima/database"
	"ahui2016.github.com/mima/util"
)

const (
	AppConfigFolder = "github-ahui2016/mima"
	dbFileName      = "db-mima.sqlite"
)

var (
	db       = new(database.DB)
	addr     = flag.String("addr", "", "Local IP address. Example: 127.0.0.1:80")
	debug    = flag.Bool("debug", false, "Switch to debug mode.")
	demo     = flag.Bool("demo", false, "Set this flag for demo.")
	dbFolder = flag.String("db", "", "Specify a folder for the database.")
)

// var sessionSecretKey = generateRandomKey()

func init() {
	flag.Parse()
	dbPath := getDBPath()
	fmt.Println("[Database]", dbPath)

	util.Panic(db.Open(dbPath))
	if *addr == "" {
		s, err := db.GetSettings()
		util.Panic(err)
		*addr = s.AppAddr
	}
}

func getDBPath() string {
	if *dbFolder != "" {
		folder, err := filepath.Abs(*dbFolder)
		util.Panic(err)
		if util.PathIsNotExist(folder) {
			log.Fatal("Not Found: " + folder)
		}
		return filepath.Join(folder, dbFileName)
	}
	userConfigDir, err := os.UserConfigDir()
	util.Panic(err)
	configFolder := filepath.Join(userConfigDir, AppConfigFolder)
	util.Panic(os.MkdirAll(configFolder, 0740))
	return filepath.Join(configFolder, dbFileName)
}
