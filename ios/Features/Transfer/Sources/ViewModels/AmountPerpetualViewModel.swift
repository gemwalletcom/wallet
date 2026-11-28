// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAmountRequest
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseSession
import struct Gemstone.GemPerpetualAmountSession
import Localization
import PrimitivesComponents

@Observable
public final class AmountPerpetualViewModel {
    private var session: GemPerpetualAmountSession

    init(session: GemPerpetualAmountSession) {
        self.session = session
    }

    var leverageSelection: SelectionState<LeverageOption>? {
        session.leverage.map {
            SelectionState(
                options: $0.options.map(LeverageOption.init(option:)),
                selected: LeverageOption(option: $0.selected),
                isEnabled: true,
                title: Localized.Perpetual.leverage,
            )
        }
    }

    var request: GemAmountRequest {
        .perpetual(session: session)
    }

    func autocloseSession(amount: String) -> GemAutocloseSession {
        session.autocloseSession(amount: amount)
    }

    func openPositionRow(amount: String) -> GemAssetItemRow {
        session.openRow(amount: amount)
    }

    func onChangeLeverage(_ leverage: UInt8) {
        session = session.onLeverage(leverage: leverage)
    }

    func updateAutoclose(takeProfit: String, stopLoss: String) {
        session = session.onAutoclose(takeProfit: takeProfit, stopLoss: stopLoss)
    }
}
