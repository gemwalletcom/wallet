// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import SwiftUI

public struct DatabaseAccess: Sendable {
    private let _dbQueue: DatabaseQueue?

    public static let notConfigured = DatabaseAccess(nil)

    public init(_ dbQueue: DatabaseQueue?) {
        _dbQueue = dbQueue
    }

    public var dbQueue: DatabaseQueue {
        guard let _dbQueue else {
            fatalError("DatabaseQueue not configured. Use .databaseQueue() modifier.")
        }
        return _dbQueue
    }
}

public extension EnvironmentValues {
    @Entry var database: DatabaseAccess = .notConfigured
}

public extension View {
    func databaseQueue(_ dbQueue: DatabaseQueue) -> some View {
        environment(\.database, DatabaseAccess(dbQueue))
    }
}
