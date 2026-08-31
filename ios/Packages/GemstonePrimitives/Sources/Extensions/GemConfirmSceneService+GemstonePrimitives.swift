// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConfirmSceneServiceProtocol
import Primitives

public extension GemConfirmSceneServiceProtocol {
    func addressName(chain: Primitives.Chain, address: String) throws -> Primitives.AddressName? {
        try addressName(chain: chain.rawValue, address: address).map { try Primitives.AddressName($0) }
    }

    func addressNames(requests: [Primitives.ChainAddress]) async throws -> [Primitives.ChainAddress: Primitives.AddressName] {
        let names = try await addressNames(requests: requests.map { $0.json() }).map { try Primitives.AddressName($0) }
        return Dictionary(uniqueKeysWithValues: names.map { (Primitives.ChainAddress(chain: $0.chain, address: $0.address), $0) })
    }

    @discardableResult
    func syncMissingAssets(for assetIds: [Primitives.AssetId]) async throws -> [Primitives.AssetId] {
        try await syncMissingAssets(assetIds: assetIds.ids).map { try Primitives.AssetId(id: $0) }
    }

    func explorerLink(chain: Primitives.Chain, address: String) -> BlockExplorerLink {
        BlockExplorerLink(addressUrl(chain: chain.rawValue, address: address))
    }

    func feeAsset(for type: TransferDataType) -> Primitives.Asset {
        do {
            return try Primitives.Asset(feeAsset(inputType: type.inputType))
        } catch {
            preconditionFailure("Undecodable transfer fee asset: \(error)")
        }
    }

    func assetIds(for type: TransferDataType) -> [Primitives.AssetId] {
        assetIds(inputType: type.inputType).compactMap { try? Primitives.AssetId(id: $0) }
    }

    func track(walletId: Primitives.WalletId, transactions: [Primitives.Transaction]) async throws {
        try await track(walletId: walletId.id, transactions: transactions.map { $0.json() })
    }
}
