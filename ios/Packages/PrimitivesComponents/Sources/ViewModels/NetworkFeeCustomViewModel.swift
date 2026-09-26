// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemCustomFeeEstimate
import struct Gemstone.GemCustomFeeSession
import GemstonePrimitives
import Localization
import Observation
import Primitives

@Observable
@MainActor
public final class NetworkFeeCustomViewModel {
    private var session: GemCustomFeeSession
    private var estimate: GemCustomFeeEstimate
    private let onSelect: @MainActor (BigInt) -> Void

    public init(session: GemCustomFeeSession, onSelect: @escaping @MainActor (BigInt) -> Void) {
        self.session = session
        estimate = session.viewState()
        self.onSelect = onSelect
    }

    public var input: String {
        get { session.input }
        set { update(session.onInput(text: newValue)) }
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeListItem: ListItemModel {
        ListItemModel(title: networkFeeTitle, subtitle: value, subtitleExtra: fiatValue)
    }

    public var networkFeeTitle: String { Localized.Transfer.networkFee }

    public var suffix: String {
        session.rows.unitType.toPrimitives().suffix(symbol: session.feeAsset.symbol)
    }

    public var placeholder: String {
        estimate.placeholder?.text() ?? ""
    }

    public var value: String? {
        estimate.fee?.amount.text()
    }

    public var fiatValue: String? {
        estimate.fee?.fiat?.text()
    }

    public var errorText: String? {
        estimate.check.errorText
    }

    public var isConfirmEnabled: Bool {
        estimate.isValid
    }

    public func sanitize(_ text: String) -> String {
        session.onInput(text: text).input
    }

    public func confirm() {
        guard let rate = estimate.rate, estimate.isValid else { return }
        onSelect(rate)
    }
}

// MARK: - Private

extension NetworkFeeCustomViewModel {
    private func update(_ session: GemCustomFeeSession) {
        self.session = session
        estimate = session.viewState()
    }
}
