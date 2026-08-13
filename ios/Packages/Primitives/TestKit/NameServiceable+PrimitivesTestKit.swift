// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives

public extension NameServiceable where Self == MockNameService {
    static func mock(nameRecord: NameRecord? = nil) -> MockNameService {
        MockNameService(nameRecord: nameRecord)
    }
}

public actor MockNameService: NameServiceable {
    let nameRecord: NameRecord?
    public private(set) var requests: [String] = []

    public init(nameRecord: NameRecord? = nil) {
        self.nameRecord = nameRecord
    }

    public func getName(name: String, chain _: String) async throws -> NameRecord? {
        requests.append(name)
        return nameRecord ?? NameRecord.mock()
    }
}
