// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.Account
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import protocol Gemstone.GemAddAssetServiceProtocol
import struct Gemstone.BlockExplorerLink
import struct Gemstone.Wallet
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemAddAssetServiceMock: GemAddAssetServiceProtocol, @unchecked Sendable {
    private let chains: [Primitives.Chain]
    private let asset: Primitives.Asset

    public init(chains: [Primitives.Chain] = [.ethereum], asset: Primitives.Asset = .mock()) {
        self.chains = chains
        self.asset = asset
    }

    public func chains(wallet _: Wallet) -> [Chain] { chains.map(\.rawValue) }
    public func defaultChain(chains: [Chain]) -> Chain? { chains.first }
    public func matchingChains(chains: [Chain], query: String) -> [Chain] { chains }
    public func tokenUrl(chain: Chain, tokenId: String) -> Gemstone.BlockExplorerLink? { nil }
    public func token(chain: Chain, address: String) async throws -> Asset { asset.map() }
    public func add(wallet: Wallet, assetId: AssetId) async throws {}
}
