// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import MarketInsight
import GemstoneServices
import SwiftUI

struct MarketsNavigationStack: View {
    @Environment(\.assetsService) private var assetsService
    @Environment(\.marketService) private var marketService

    var body: some View {
        MarketsScene(
            model: MarketsSceneViewModel(
                service: marketService,
                assetsService: assetsService,
            ),
        )
    }
}
