// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import WidgetKit

@main
struct GemPriceWidgetBundle: WidgetBundle {
    var body: some Widget {
        SmallPriceWidget()
        MediumPriceWidget()
    }
}
