// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.fiatProviderName
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public extension FiatProviderName {
    var image: Image {
        switch self {
        case .moonPay: Images.Fiat.moonpay
        case .transak: Images.Fiat.transak
        case .banxa: Images.Fiat.banxa
        case .mercuryo: Images.Fiat.mercuryo
        case .paybis: Images.Fiat.paybis
        case .flashnet: Images.Fiat.cashapp
        }
    }

    var displayName: String {
        fiatProviderName(provider: map())
    }
}
