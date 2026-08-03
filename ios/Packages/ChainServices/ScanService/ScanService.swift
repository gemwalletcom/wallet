// Copyright (c). Gem Wallet. All rights reserved.

import GemAPI
import Primitives

public struct ScanService: Sendable {
    private let apiService: any GemAPIScanService

    public init(apiService: any GemAPIScanService) {
        self.apiService = apiService
    }

    public func getScanTransaction(chain _: Primitives.Chain, input: TransactionPreloadInput) async -> ScanTransaction? {
        let originAssetId = input.inputType.assetIds.first ?? input.inputType.chain.assetId
        let targetAssetId = input.inputType.assetIds.last ?? originAssetId
        let website: String? = switch input.inputType {
        case let .generic(_, app, _): app.url
        default: nil
        }
        let payload = ScanTransactionPayload(
            origin: ScanAddressTarget(
                assetId: originAssetId,
                address: input.senderAddress,
            ),
            target: ScanAddressTarget(
                assetId: targetAssetId,
                address: input.destinationAddress,
            ),
            website: website,
            type: input.inputType.transactionType,
        )
        return try? await apiService.getScanTransaction(payload: payload)
    }
}
