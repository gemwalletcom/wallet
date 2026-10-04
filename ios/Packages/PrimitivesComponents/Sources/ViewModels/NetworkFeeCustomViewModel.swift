// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemCustomFeeEstimate
import struct Gemstone.GemCustomFeeField
import struct Gemstone.GemCustomFeeFieldState
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
    private let onSelect: @MainActor (GemConfirmFeeSelection) -> Void

    public init(
        session: GemCustomFeeSession,
        onSelect: @escaping @MainActor (GemConfirmFeeSelection) -> Void,
    ) {
        self.session = session
        estimate = session.viewState()
        self.onSelect = onSelect
    }

    var input: String {
        get { session.rate.input }
        set { update(session.onInput(text: newValue)) }
    }

    var baseFeeInput: String {
        get { session.baseFee?.input ?? "" }
        set { update(session.onBaseFeeInput(text: newValue)) }
    }

    public var title: String { Localized.FeeRate.custom }
    public var networkFeeListItem: ListItemModel {
        ListItemModel(title: networkFeeTitle, subtitle: value, subtitleExtra: fiatValue)
    }

    var networkFeeTitle: String { Localized.Transfer.networkFee }

    var rateField: NetworkFeeCustomFieldModel {
        fieldModel(session.rate, estimate.rate)
    }

    var baseFeeField: NetworkFeeCustomFieldModel? {
        guard let field = session.baseFee, let state = estimate.baseFee else { return nil }
        return fieldModel(field, state)
    }

    var suffix: String {
        session.rows.unitType.toPrimitives().suffix(symbol: session.feeAsset.symbol)
    }

    var value: String? {
        estimate.fee?.amount.text()
    }

    public var fiatValue: String? {
        estimate.fee?.fiat?.text()
    }

    var isConfirmEnabled: Bool {
        estimate.selection != nil
    }

    public func sanitize(_ text: String) -> String {
        session.onInput(text: text).rate.input
    }

    func confirm() {
        guard let selection = estimate.selection else { return }
        onSelect(selection)
    }
}

// MARK: - Private

extension NetworkFeeCustomViewModel {
    private func update(_ session: GemCustomFeeSession) {
        self.session = session
        estimate = session.viewState()
    }

    private func fieldModel(_ field: GemCustomFeeField, _ state: GemCustomFeeFieldState) -> NetworkFeeCustomFieldModel {
        NetworkFeeCustomFieldModel(
            title: field.title.text,
            placeholder: state.placeholder?.text() ?? "",
            errorText: state.check.errorText,
        )
    }
}
