// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.AddressName
public import typealias Gemstone.Chain
public import typealias Gemstone.Currency
public import protocol Gemstone.GemConfirmationProtocol
public import enum Gemstone.GemConfirmError
public import struct Gemstone.GemConfirmErrorInfo
public import struct Gemstone.GemConfirmHeader
public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmLoadOptions
public import enum Gemstone.GemConfirmRowContent
public import struct Gemstone.GemConfirmScreen
public import enum Gemstone.GemConfirmSection
public import struct Gemstone.GemConfirmViewState
public import struct Gemstone.GemFeeRateRows
public import enum Gemstone.GemKeystoreAuthentication
public import enum Gemstone.GemListRow
public import struct Gemstone.GemNetworkFeeScreen
public import struct Gemstone.GemNumberFormat
public import enum Gemstone.GemSubmitResult
public import struct Gemstone.GemTransferData
public import typealias Gemstone.PerpetualModifyConfirmData
import func Gemstone.confirmErrorInfo
import enum Gemstone.GemConfirmDetails
import struct Gemstone.GemConfirmFeeRow
import func Gemstone.perpetualConfirmDetails
import func Gemstone.swapQuoteDetails
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

    public var headerValue: GemConfirmHeader = .mock()

    public func screen() -> GemConfirmScreen {
        .mock()
    }

    public func loadOptions() -> GemConfirmLoadOptions {
        GemConfirmLoadOptions(feeSelection: .priority(priority: .normal), feeAssetId: nil, assetId: nil)
    }

    public func header(screen _: GemConfirmScreen) -> GemConfirmHeader {
        headerValue
    }

    public func viewState(screen: GemConfirmScreen) -> GemConfirmViewState {
        let simulation = loaded?.simulation.simulation
        let warnings = loaded?.simulation.warnings ?? warnings
        let sections: [GemConfirmSection?] = [
            .header,
            .details(rows: rowContents(addressName: loaded?.addressName)),
            warnings.isEmpty ? nil : .warnings(rows: warnings),
            simulation.flatMap { $0.primaryFields.isEmpty ? nil : .payload(primary: $0.primaryFields, secondary: $0.secondaryFields) },
            simulation.flatMap { $0.balanceChanges.isEmpty ? nil : .balanceChanges(changes: $0.balanceChanges) },
            transfer().verification() == nil ? .networkFee : .verification,
            screen.failure.flatMap { $0.stage == .load ? .error(error: $0.error) : nil },
        ]
        return GemConfirmViewState(
            button: screen.button(),
            feeRow: feeRow(screen: screen),
            details: details(),
            title: transfer().title(),
            verification: transfer().verification(),
            authentication: authenticationValue,
            sections: sections.compactMap(\.self),
        )
    }

    private func details() -> GemConfirmDetails? {
        switch transfer().inputType {
        case let .swap(fromAsset, toAsset, swapData):
            .swap(details: swapQuoteDetails(quote: swapData.quote, fromAsset: fromAsset, toAsset: toAsset, fromPrice: nil, toPrice: nil, currency: getCurrency()))
        case let .perpetual(_, perpetualType):
            perpetualConfirmDetails(perpetualType: perpetualType).map { .perpetual(details: $0) }
        case .transfer, .deposit, .withdrawal, .stake, .tokenApprove, .generic, .payment, .transferNft, .account, .earn:
            nil
        }
    }

    private func feeRow(screen: GemConfirmScreen) -> GemConfirmFeeRow {
        let value = screen.feeValue(load: loaded)
        let isUnavailable = if case .unavailable = value {
            true
        } else {
            false
        }
        return GemConfirmFeeRow(
            title: .networkFee,
            value: value,
            info: .networkFee(asset: loaded?.feeAsset ?? transfer().feeAsset()),
            opensDetails: feeRates != nil && !isUnavailable,
        )
    }

    public func networkFeeScreen(format _: GemNumberFormat) -> GemNetworkFeeScreen? {
        feeRates.map { GemNetworkFeeScreen.mock(fee: loaded?.fee?.formatted, additionalFees: loaded?.fee?.additionalFees ?? [], rates: $0) }
    }

    public func transfer() -> GemTransferData {
        selected ?? (loaded ?? initialState).transfer
    }

    public func state() async throws -> GemConfirmLoad {
        let state = loaded ?? initialState
        loaded = state
        return state
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
