// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives

public extension NameServiceable where Self == MockNameService {
    static func mock(nameRecord: NameRecord? = nil) -> MockNameService {
        MockNameService(nameRecord: nameRecord)
    }
}

public struct MockNameService: NameServiceable {
    let nameRecord: NameRecord?

    public init(nameRecord: NameRecord? = nil) {
        self.nameRecord = nameRecord
    }

    public func getName(name _: String, chain _: String) async throws -> NameRecord? {
        nameRecord ?? NameRecord.mock()
    }
}
