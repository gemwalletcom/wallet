// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import class Gemstone.GemCustomFee
import struct Gemstone.GemFeeAmount
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
import Localization
import Observation
import Primitives

@Observable
@MainActor
public final class NetworkFeeCustomViewModel {
    private let chain: Chain
    private let feeAsset: Asset
    private let rows: GemFeeRateRows
    private let baseFee: BigInt?
    private let onSelect: @MainActor (BigInt) -> Void
    private let display: (BigInt) -> GemFeeAmount

    public var input: String = ""

    public init(
        chain: Chain,
        feeAsset: Asset,
        rows: GemFeeRateRows,
        baseFee: BigInt?,
        initialRate: BigInt?,
        onSelect: @escaping @MainActor (BigInt) -> Void,
        display: @escaping (BigInt) -> GemFeeAmount,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        self.rows = rows
        self.baseFee = baseFee
        self.onSelect = onSelect
        self.display = display
        input = initialRate.flatMap { NumberInput.format().inputText(value: $0.description, decimals: rows.unitDecimals) } ?? ""
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeListItem: ListItemModel {
        ListItemModel(title: networkFeeTitle, subtitle: value, subtitleExtra: fiatValue)
    }

    public var networkFeeTitle: String { Localized.Transfer.networkFee }

    public var suffix: String {
        rows.unitType.toPrimitives().suffix(symbol: feeAsset.symbol)
    }

    public var placeholder: String {
        estimate.placeholder()?.text() ?? ""
    }

    public var value: String? {
        feeAmount.map { display($0).amount.text() }
    }

    public var fiatValue: String? {
        feeAmount.flatMap { display($0).fiat?.text() }
    }

    public var errorText: String? {
        switch estimate.check() {
        case let .belowMinimum(rate): Localized.Common.minimumValue(rate.text)
        case let .overMaximum(rate): Localized.Common.maximumValue(rate.text)
        case .valid: nil
        }
    }

    public var isConfirmEnabled: Bool {
        estimate.isValid()
    }

    public func sanitize(_ text: String) -> String {
        NumberInput.format().sanitize(input: text, maximumFractionDigits: rows.unitDecimals, maximumIntegerDigits: nil)
    }

    public func confirm() {
        let estimate = estimate
        guard let rate = estimate.rate(), estimate.isValid() else { return }
        onSelect(rate)
    }

    private var estimate: GemCustomFee {
        GemCustomFee.estimate(
            chain: chain.rawValue,
            input: input,
            format: NumberInput.format(),
            rows: rows,
            loadedFee: baseFee ?? .zero,
        )
    }

    private var feeAmount: BigInt? {
        baseFee.map { _ in estimate.feeValue() }
    }
}
