// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.BlockExplorerLink
public import typealias Gemstone.Chain
public import typealias Gemstone.Currency
public import enum Gemstone.GemAcquireAssetFlow
public import class Gemstone.GemAssetConfigService
public import struct Gemstone.GemAutocloseSummary
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmScreen
public import protocol Gemstone.GemConfirmSessionProtocol
public import enum Gemstone.GemExecuteResult
public import enum Gemstone.GemKeystoreAuthentication
public import typealias Gemstone.PerpetualModifyConfirmData
import GemstonePrimitivesTestKit
import Primitives

public final class GemConfirmSessionMock: GemConfirmSessionProtocol, @unchecked Sendable {
    private let initialState: GemConfirmLoad
    private let loadResult: Result<GemConfirmLoad, any Error>
    private let executeResult: Result<GemExecuteResult, any Error>
    private let authenticationValue: GemKeystoreAuthentication
    private let assetConfig = GemAssetConfigService()
    private var loaded: GemConfirmLoad?
    public private(set) var loadOptions: [GemConfirmLoadOptions] = []
    public var onLoad: (@MainActor () -> Void)?

    public init(
        state: GemConfirmLoad = .mock(),
        load: Result<GemConfirmLoad, any Error> = .success(.mock()),
        execute: Result<GemExecuteResult, any Error> = .success(.signed(data: [])),
        authentication: GemKeystoreAuthentication = .none,
    ) {
        initialState = state
        loadResult = load
        executeResult = execute
        authenticationValue = authentication
    }

    public func screen() -> GemConfirmScreen {
        GemConfirmScreen(phase: .loading, amountFailed: false, hasCriticalWarning: false, failure: nil)
    }

    public func state() async throws -> GemConfirmLoad {
        loaded ?? initialState
    }

    public func load(options: GemConfirmLoadOptions) async throws -> GemConfirmLoad {
        loadOptions.append(options)
        await onLoad?()
        loaded = try loadResult.get()
        return try loadResult.get()
    }

    public func execute() async throws -> GemExecuteResult {
        try executeResult.get()
    }

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.rawValue
    }

    public func authentication() -> GemKeystoreAuthentication {
        authenticationValue
    }

    public func addressUrl(chain: Chain, address: String) -> BlockExplorerLink {
        BlockExplorerLink(name: "Explorer", link: "https://explorer.test/\(chain)/\(address)")
    }

    public func acquireAssetFlow(chain: Chain) -> GemAcquireAssetFlow {
        assetConfig.acquireFlow(chain: chain)
    }

    public func insufficientNetworkFeeBuyAmount() -> Int32 {
        Self.networkFeeBuyAmount
    }

    public func autocloseSummary(data _: PerpetualModifyConfirmData) -> GemAutocloseSummary? {
        nil
    }

    public static let networkFeeBuyAmount: Int32 = 10
}
