// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.Account
import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetBasic
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import typealias Gemstone.Currency
import enum Gemstone.GemAssetAction
import protocol Gemstone.GemAssetSelectionServiceProtocol
import struct Gemstone.GemNftEntry
import enum Gemstone.GemSearchScope
import struct Gemstone.GemSelectAssetFlow
import enum Gemstone.GemSelectAssetType
import struct Gemstone.GemSelectAssetWalletFlow
import struct Gemstone.GemWalletSearchInput
import struct Gemstone.GemWalletSearchLimits
import struct Gemstone.GemWalletSearchView
import typealias Gemstone.NftData
import struct Gemstone.Wallet
import func Gemstone.walletSearchState
import enum Gemstone.WalletType
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

    public var tokensSupported = true
    public var nftSearchItems: [GemNftEntry] = []
    public var filterChainsResult: [Gemstone.Chain] = []
    public private(set) var pinnedPerpetuals: [(perpetualId: String, pinned: Bool)] = []

    public func flow(selectType: GemSelectAssetType) -> GemSelectAssetFlow {
        selectType.flow()
    }

    public func walletSearchLimits(query _: String) -> GemWalletSearchLimits {
        GemWalletSearchLimits(assets: 12, fetch: 13, perpetuals: 3, nfts: 3, results: 100)
    }

    public func walletSearchView(input: GemWalletSearchInput) -> GemWalletSearchView {
        let limits = walletSearchLimits(query: input.query)
        let showsRecents = flow(selectType: .walletSearch).showsRecents(isSearching: input.query.isNotEmpty, hasRecents: input.counts.recents > 0)
        var counts = input.counts
        if !showsRecents {
            counts.recents = 0
        }
        if !perpetualsShown {
            counts.perpetuals = 0
            counts.pinnedPerpetuals = 0
        }
        return GemWalletSearchView(
            state: walletSearchState(counts: counts, isLoading: input.isLoading),
            limits: limits,
            hasMoreAssets: counts.assets > limits.assets,
            hasMorePerpetuals: counts.perpetuals > limits.perpetuals,
            hasMoreNfts: counts.nfts > limits.nfts,
            showsAddToken: walletFlow(selectType: .walletSearch, wallet: input.wallet).showsAddToken,
        )
    }

    public var perpetualsShown = true

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.toGem()
    }

    public func showPerpetuals(walletType _: Gemstone.WalletType, chains _: [Gemstone.Chain]) -> Bool {
        perpetualsShown
    }

    public func searchCollections(data _: [NftData], query _: String) -> [GemNftEntry] {
        nftSearchItems
    }

    public func walletFlow(selectType: GemSelectAssetType, wallet: Gemstone.Wallet) -> GemSelectAssetWalletFlow {
        let flow = selectType.flow()
        let hasChains = filterChainsResult.isNotEmpty
        return GemSelectAssetWalletFlow(
            flow: flow,
            chains: filterChainsResult,
            showsAddToken: flow.addCustomToken && tokensSupported && hasChains,
            showsChainFilter: flow.chainFilter && wallet.walletType == .multicoin && hasChains,
        )
    }

    public func search(query _: String, scope _: GemSearchScope) async throws -> Bool {
        if let error {
            throw error
        }
        return true
    }

    public func searchKey(query: String, scope: GemSearchScope) -> String {
        let query = query.trimmingCharacters(in: .whitespacesAndNewlines)
        switch scope {
        case .all: return query
        case let .list(id): return query.isEmpty ? "tag:\(id)" : query
        }
    }

    public func setAssetPinned(assetId: AssetId, pinned: Bool) async throws {
        onSetAssetPinned?(assetId, pinned)
    }

    public func setPerpetualPinned(perpetualId: String, pinned: Bool) async throws {
        pinnedPerpetuals.append((perpetualId, pinned))
    }

    public func searchAssets(query _: String) async throws -> [AssetBasic] {
        if let error {
            throw error
        }
        return assets
    }

    public func setAssetsEnabled(assetIds: [AssetId], enabled: Bool) async throws {
        onSetAssetsEnabled?(assetIds, enabled)
        if let error {
            throw error
        }
    }

    public func addRecent(action _: GemAssetAction, asset _: Asset) async throws {}

    public func setPriceAlert(assetId _: AssetId, enabled _: Bool) async throws {}
}
