// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import Perpetuals
import Primitives
import PrimitivesComponents

public enum AmountSheetType: Identifiable {
    case infoAction(InfoSheetType)
    case fiatConnect(assetAddress: AssetAddress, wallet: Wallet)
    case leverageSelector(selection: SelectionState<LeverageOption>)
    case autoclose(AutocloseOpenData)

    public var id: String {
        switch self {
        case let .infoAction(type): "info-action-\(type.id)"
        case .fiatConnect: "fiat-connect"
        case .leverageSelector: "leverage-selector"
        case .autoclose: "autoclose"
        }
    }
}
