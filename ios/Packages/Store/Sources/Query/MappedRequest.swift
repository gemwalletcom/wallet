// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB

public struct MappedRequest<Base: DatabaseQueryable, Value: Equatable & Sendable>: DatabaseQueryable {
    public var base: Base
    private let transform: @Sendable (Base.Value) -> Value

    public init(_ base: Base, transform: @escaping @Sendable (Base.Value) -> Value) {
        self.base = base
        self.transform = transform
    }

    public func fetch(_ db: Database) throws -> Value {
        transform(try base.fetch(db))
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.base == rhs.base
    }
}
