// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import SwiftUI

public struct DatabaseAccess: Sendable {
    private let _dbPool: DatabasePool?

    public static let notConfigured = DatabaseAccess(nil)

    public init(_ dbPool: DatabasePool?) {
        _dbPool = dbPool
    }

    public var dbPool: DatabasePool {
        guard let _dbPool else {
            fatalError("DatabasePool not configured. Use .databasePool() modifier.")
        }
        return _dbPool
    }
}

public extension EnvironmentValues {
    @Entry var database: DatabaseAccess = .notConfigured
}

public extension View {
    func databasePool(_ dbPool: DatabasePool) -> some View {
        environment(\.database, DatabaseAccess(dbPool))
    }
}
