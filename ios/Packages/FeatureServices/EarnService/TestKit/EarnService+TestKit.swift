// Copyright (c). Gem Wallet. All rights reserved.

import EarnService
import Primitives
import PrimitivesTestKit

public final class MockEarnService: EarnDataProvidable, @unchecked Sendable {
    public init() {}

    public func getEarnData(assetId _: AssetId, address _: String, value _: String, earnType _: EarnType) async throws -> ContractCallData {
        .mock()
    }
}

public extension MockEarnService {
    static func mock() -> MockEarnService {
        MockEarnService()
    }
}
