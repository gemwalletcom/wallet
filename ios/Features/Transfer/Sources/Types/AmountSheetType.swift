// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemInfoSheet
import InfoSheet
import Primitives
import PrimitivesComponents

public enum AmountSheetType: Identifiable {
    case infoAction(GemInfoSheet)
    case fiatConnect(assetAddress: AssetAddress, wallet: Wallet)
    case leverageSelector(selection: SelectionState<LeverageOption>)
    case autoclose(AutocloseOpenData)

    public var id: String {
        switch self {
        case let .infoAction(sheet): "info-action-\(sheet.hashValue)"
        case .fiatConnect: "fiat-connect"
        case .leverageSelector: "leverage-selector"
        case .autoclose: "autoclose"
        }
    }
}
