// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.AddressName
public import typealias Gemstone.Chain
public import typealias Gemstone.Currency
public import enum Gemstone.GemAcquireAssetFlow
public import protocol Gemstone.GemConfirmationProtocol
public import enum Gemstone.GemConfirmError
public import struct Gemstone.GemConfirmErrorInfo
public import enum Gemstone.GemConfirmHeader
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmMetadata
public import enum Gemstone.GemConfirmRowContent
public import struct Gemstone.GemConfirmScreen
public import enum Gemstone.GemKeystoreAuthentication
public import enum Gemstone.GemListRow
public import enum Gemstone.GemSubmitResult
public import struct Gemstone.GemSwapPairSelection
public import struct Gemstone.GemTransferData
public import typealias Gemstone.PerpetualModifyConfirmData
import func Gemstone.confirmErrorInfo
import GemstonePrimitivesTestKit
import Primitives

public final class GemConfirmationMock: GemConfirmationProtocol, @unchecked Sendable {
    private let initialState: GemConfirmLoad
    private let loadResult: Result<GemConfirmLoad, any Error>
    private let executeResult: Result<GemSubmitResult, any Error>
    private let authenticationValue: GemKeystoreAuthentication
    private let rows: (Gemstone.AddressName?) -> [GemConfirmRowContent]
    private let acquireFlow: GemAcquireAssetFlow
    private let selection: GemTransferData?
    private var loaded: GemConfirmLoad?
    private var selected: GemTransferData?
    public private(set) var requestedOptions: [GemConfirmLoadOptions] = []
    public var onLoad: (@MainActor () -> Void)?

    public init(
        state: GemConfirmLoad = .mock(),
        load: Result<GemConfirmLoad, any Error> = .success(.mock()),
        execute: Result<GemSubmitResult, any Error> = .success(.signed(data: [], warning: nil)),
        authentication: GemKeystoreAuthentication = .none,
        rows: @escaping (Gemstone.AddressName?) -> [GemConfirmRowContent] = { _ in [] },
        acquireFlow: GemAcquireAssetFlow = .fiat,
        selection: GemTransferData? = nil,
    ) {
        initialState = state
        loadResult = load
        executeResult = execute
        authenticationValue = authentication
        self.rows = rows
        self.acquireFlow = acquireFlow
        self.selection = selection
    }

    public var headerValue: GemConfirmHeader = .transaction(header: .symbol(asset: Asset.mock().toGem()))

    public func screen() -> GemConfirmScreen {
        .mock()
    }

    public func loadOptions() -> GemConfirmLoadOptions {
        GemConfirmLoadOptions(feeSelection: .priority(priority: transfer().defaultFeePriority()), feeAssetId: nil, assetId: nil)
    }

    public func header(load _: GemConfirmLoad?) -> GemConfirmHeader {
        headerValue
    }

    public func transfer() -> GemTransferData {
        selected ?? (loaded ?? initialState).transfer
    }

    public func state() async throws -> GemConfirmLoad {
        loaded ?? initialState
    }

    public func load(options: GemConfirmLoadOptions) async throws -> GemConfirmLoad {
        requestedOptions.append(options)
        await onLoad?()
        if options.assetId != nil {
            selected = selection
        }
        loaded = try loadResult.get()
        return try loadResult.get()
    }

    public func submit() async throws -> GemSubmitResult {
        try executeResult.get()
    }

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.toGem()
    }

    public func authentication() -> GemKeystoreAuthentication {
        authenticationValue
    }

    public func rowContents(addressName: Gemstone.AddressName?) -> [GemConfirmRowContent] {
        rows(addressName)
    }

    public func acquireAssetFlow(chain _: Chain) -> GemAcquireAssetFlow {
        acquireFlow
    }

    public func acquireSwapPair(feeAssetId: String?, assetId: String) -> GemSwapPairSelection {
        GemSwapPairSelection(payAssetId: feeAssetId, receiveAssetId: assetId)
    }

    public func errorInfo(error: GemConfirmError, metadata: GemConfirmMetadata?) -> GemConfirmErrorInfo? {
        confirmErrorInfo(error: error, prices: metadata?.prices ?? [], currency: getCurrency())
    }

    public func insufficientNetworkFeeBuyAmount() -> Int32 {
        Self.networkFeeBuyAmount
    }

    public func autocloseRow(data _: PerpetualModifyConfirmData) -> GemListRow? {
        nil
    }

    public static let networkFeeBuyAmount: Int32 = 10
}
