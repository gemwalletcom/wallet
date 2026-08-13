// Copyright (c). Gem Wallet. All rights reserved.

import EarnService
import Primitives
import PrimitivesTestKit

public final class MockEarnService: EarnDataProvidable, EarnPositionsUpdatable, @unchecked Sendable {
    public init() {}

    public func getEarnData(assetId _: AssetId, address _: String, value _: String, earnType _: EarnType) async throws -> ContractCallData {
        .mock()
    }

    public func update(walletId _: WalletId, assetId _: AssetId, address _: String) async throws {}
}

public extension MockEarnService {
    static func mock() -> MockEarnService {
        MockEarnService()
    }
}

public extension EarnPositionsUpdatable where Self == MockEarnService {
    static func mock() -> MockEarnService {
        MockEarnService()
    }
}
