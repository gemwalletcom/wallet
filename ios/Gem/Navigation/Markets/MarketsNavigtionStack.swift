// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import MarketInsight
import PriceService
import SwiftUI

struct MarketsNavigationStack: View {
    @Environment(\.assetsService) private var assetsService

    var body: some View {
        MarketsScene(
            model: MarketsSceneViewModel(
                service: MarketService(),
                assetsService: assetsService,
            ),
        )
    }
}
