// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import os
import Primitives

public struct DB: Sendable {
    public let dbPool: DatabasePool

    public init(
        fileName: String = "db.sqlite",
        configuration: GRDB.Configuration = DB.defaultConfiguration,
    ) {
        do {
            let fileMigrator = FileMigrator()
            let databaseURL = try fileMigrator.migrate(
                name: fileName,
                fromDirectory: .documentDirectory,
                toDirectory: .applicationSupportDirectory,
                isDirectory: false,
            )
            dbPool = try DatabasePool(path: databaseURL.path(percentEncoded: false), configuration: configuration)
        } catch {
            os_log(
                "DB Initialization error: %{public}@",
                log: .default,
                type: .fault,
                error.localizedDescription,
            )
            fatalError("DB Initialization error: \(error)")
        }

        do {
            var migrations = Migrations()
            try migrations.run(dbPool)
            if !isRunningTests {
                try migrations.runChanges(dbPool)
            }
        } catch {
            os_log(
                "DB Migration error: %{public}@",
                log: .default,
                type: .fault,
                error.localizedDescription,
            )
            fatalError("DB Migration error: \(error)")
        }
    }

    public static let defaultConfiguration: GRDB.Configuration = {
        var config = GRDB.Configuration()
        config.maximumReaderCount = 12
        config.prepareDatabase { db in
            try db.execute(sql: "PRAGMA synchronous = FULL")
        }
        #if DEBUG
            config.publicStatementArguments = true
        #endif
        return config
    }()
}

var isRunningTests: Bool {
    ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] != nil
}
