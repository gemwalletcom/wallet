// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.Account
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import protocol Gemstone.GemAddAssetServiceProtocol
import struct Gemstone.GemAddAssetSession
import struct Gemstone.GemListSection
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

    public func newSession(chain: Gemstone.Chain?) -> GemAddAssetSession {
        GemAddAssetSession(chain: chain, address: "", asset: nil, isLoading: false, failed: false)
    }

    public func chains(wallet _: Wallet) -> [Chain] {
        chains.map(\.rawValue)
    }

    public func defaultChain(chains: [Chain]) -> Chain? {
        chains.first
    }

    public func sections(session _: GemAddAssetSession) -> [GemListSection] {
        []
    }

    public func token(chain _: Chain, address _: String) async throws -> Asset {
        asset.toGem()
    }

    public func add(wallet _: Wallet, assetId _: AssetId) async throws {}
}
