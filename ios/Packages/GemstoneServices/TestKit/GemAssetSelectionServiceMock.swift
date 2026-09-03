// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetBasic
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import typealias Gemstone.Currency
import enum Gemstone.GemAssetAction
import protocol Gemstone.GemAssetSelectionServiceProtocol
import enum Gemstone.GemNftSearchItem
import typealias Gemstone.NftData
import enum Gemstone.GemSearchScope
import Primitives

public final class GemAssetSelectionServiceMock: GemAssetSelectionServiceProtocol, @unchecked Sendable {
    private let assets: [AssetBasic]
    private let error: Error?
    private let onSetAssetsEnabled: (@Sendable ([AssetId], Bool) -> Void)?
    private let onSetAssetPinned: (@Sendable (AssetId, Bool) -> Void)?

    public init(
        assets: [AssetBasic] = [],
        error: Error? = nil,
        onSetAssetsEnabled: (@Sendable ([AssetId], Bool) -> Void)? = nil,
        onSetAssetPinned: (@Sendable (AssetId, Bool) -> Void)? = nil,
    ) {
        self.assets = assets
        self.error = error
        self.onSetAssetsEnabled = onSetAssetsEnabled
        self.onSetAssetPinned = onSetAssetPinned
    }

    public var perpetualsShown = true
    public var tokensSupported = true
    public var nftSearchItems: [GemNftSearchItem] = []
    public var filterChainsResult: [Gemstone.Chain] = []
    public private(set) var pinnedPerpetuals: [(perpetualId: String, pinned: Bool)] = []

    public func currency() -> Currency {
        Primitives.Currency.usd.rawValue
    }

    public func showPerpetuals() -> Bool {
        perpetualsShown
    }

    public func searchCollections(data _: [NftData], query _: String) -> [GemNftSearchItem] {
        nftSearchItems
    }

    public func supportsTokens() -> Bool {
        tokensSupported
    }

    public func filterChains() throws -> [Gemstone.Chain] {
        filterChainsResult
    }

    public func search(query _: String, scope _: GemSearchScope) async throws -> Bool {
        if let error { throw error }
        return true
    }

    public func setAssetPinned(assetId: AssetId, pinned: Bool) async throws {
        onSetAssetPinned?(assetId, pinned)
    }

    public func setPerpetualPinned(perpetualId: String, pinned: Bool) async throws {
        pinnedPerpetuals.append((perpetualId, pinned))
    }

    public func searchAssets(query _: String) async throws -> [AssetBasic] {
        if let error { throw error }
        return assets
    }

    public func setAssetsEnabled(assetIds: [AssetId], enabled: Bool) async throws {
        onSetAssetsEnabled?(assetIds, enabled)
    }

    public func addRecent(action _: GemAssetAction, asset _: Asset) async throws {}

    public func setPriceAlert(assetId _: AssetId, enabled _: Bool) async throws {}
}
