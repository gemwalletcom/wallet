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
import enum Gemstone.GemSearchScope
import struct Gemstone.GemSelectAssetFlow
import enum Gemstone.GemSelectAssetType
import struct Gemstone.GemSelectAssetWalletFlow
import struct Gemstone.GemToast
import struct Gemstone.GemWalletSearchInput
import struct Gemstone.GemWalletSearchLimits
import struct Gemstone.GemWalletSearchResultsInput
import struct Gemstone.GemWalletSearchResultsView
import struct Gemstone.GemWalletSearchView
import struct Gemstone.Wallet
import GemstonePrimitivesTestKit
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
    public var filterChainsResult: [Gemstone.Chain] = []
    public private(set) var pinnedPerpetuals: [(perpetualId: PerpetualId, pinned: Bool)] = []

    public func flow(selectType: GemSelectAssetType) -> GemSelectAssetFlow {
        selectType.flow()
    }

    public func walletSearchLimits(query _: String) -> GemWalletSearchLimits {
        GemWalletSearchLimits(fetch: 13, results: 100)
    }

    public var searchView: GemWalletSearchView = .mock()
    public private(set) var searchInputs: [GemWalletSearchInput] = []

    public func walletSearchView(input: GemWalletSearchInput) -> GemWalletSearchView {
        searchInputs.append(input)
        return searchView
    }

    public var resultsView: GemWalletSearchResultsView = .mock()
    public private(set) var resultsInputs: [GemWalletSearchResultsInput] = []

    public func walletSearchResultsView(input: GemWalletSearchResultsInput) -> GemWalletSearchResultsView {
        resultsInputs.append(input)
        return resultsView
    }

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.toGem()
    }

    public func walletFlow(selectType: GemSelectAssetType, wallet: Gemstone.Wallet) -> GemSelectAssetWalletFlow {
        let flow = selectType.flow()
        let hasChains = filterChainsResult.isNotEmpty
        let showsAddToken = flow.addCustomToken && tokensSupported && hasChains
        return GemSelectAssetWalletFlow(
            flow: flow,
            chains: filterChainsResult,
            showsAddToken: showsAddToken,
            showsChainFilter: flow.chainFilter && wallet.walletType == .multicoin && hasChains,
            emptyState: .mock(actions: showsAddToken ? [.addCustomToken] : []),
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

    public func setAssetPinned(asset: Asset, pinned: Bool) async throws -> GemToast {
        onSetAssetPinned?(asset.id, pinned)
        return GemToast(text: .pinned(name: asset.name, pinned: pinned), icon: pinned ? .pin : .unpin)
    }

    public func setPerpetualPinned(perpetualId: PerpetualId, name: String, pinned: Bool) async throws -> GemToast {
        pinnedPerpetuals.append((perpetualId, pinned))
        return GemToast(text: .pinned(name: name, pinned: pinned), icon: pinned ? .pin : .unpin)
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
}
