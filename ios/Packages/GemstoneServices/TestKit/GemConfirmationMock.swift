// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.AddressName
public import typealias Gemstone.Chain
public import typealias Gemstone.Currency
public import protocol Gemstone.GemConfirmationProtocol
public import enum Gemstone.GemConfirmError
public import struct Gemstone.GemConfirmErrorInfo
public import enum Gemstone.GemConfirmHeader
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmLoadOptions
public import enum Gemstone.GemConfirmRowContent
public import struct Gemstone.GemConfirmScreen
public import struct Gemstone.GemConfirmViewState
public import struct Gemstone.GemFeeRateRows
public import enum Gemstone.GemKeystoreAuthentication
public import enum Gemstone.GemListRow
public import enum Gemstone.GemSubmitResult
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
    private let selection: GemTransferData?
    private let feeRates: GemFeeRateRows?
    private let warnings: [GemListRow]
    private var loaded: GemConfirmLoad?
    private var selected: GemTransferData?
    public private(set) var requestedOptions: [GemConfirmLoadOptions] = []
    public var onLoad: (@MainActor () -> Void)?

    public init(
        state: GemConfirmLoad = .mock(),
        load: Result<GemConfirmLoad, any Error> = .success(.mock()),
        execute: Result<GemSubmitResult, any Error> = .success(.signed(data: [], message: nil)),
        authentication: GemKeystoreAuthentication = .none,
        rows: @escaping (Gemstone.AddressName?) -> [GemConfirmRowContent] = { _ in [] },
        selection: GemTransferData? = nil,
        feeRates: GemFeeRateRows? = nil,
        warnings: [GemListRow] = [],
    ) {
        initialState = state
        loadResult = load
        executeResult = execute
        authenticationValue = authentication
        self.rows = rows
        self.selection = selection
        self.feeRates = feeRates
        self.warnings = warnings
    }

    public var headerValue: GemConfirmHeader = .transaction(header: .symbol(asset: Asset.mock().toGem()))

    public func screen() -> GemConfirmScreen {
        .mock()
    }

    public func loadOptions() -> GemConfirmLoadOptions {
        GemConfirmLoadOptions(feeSelection: .priority(priority: .normal), feeAssetId: nil, assetId: nil)
    }

    public func header(screen _: GemConfirmScreen) -> GemConfirmHeader {
        headerValue
    }

    public func viewState(screen: GemConfirmScreen, addressName: Gemstone.AddressName?) -> GemConfirmViewState {
        GemConfirmViewState(
            button: screen.button(),
            feeRow: screen.feeRow(),
            feeRates: feeRateRows(),
            rowContents: rowContents(addressName: addressName),
            simulationWarnings: loaded?.simulation.warnings ?? warnings,
        )
    }

    public func feeRateRows() -> GemFeeRateRows? {
        feeRates
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

    public func errorInfo(error: GemConfirmError) -> GemConfirmErrorInfo? {
        let state = loaded ?? initialState
        return confirmErrorInfo(
            error: error,
            prices: state.metadata.prices,
            currency: getCurrency(),
            inputAssetId: transfer().inputAsset().id,
            feeAssetId: state.feeAsset.id,
        )
    }

    public func autocloseRow(data _: PerpetualModifyConfirmData) -> GemListRow? {
        nil
    }
}
