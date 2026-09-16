// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.Latency
import Localization
import Style
import SwiftUI

struct LatencyViewModel {
    private let latency: Latency

    init(latency: Latency) {
        self.latency = latency
    }

    var title: String {
        Localized.Common.latencyInMs(value)
    }

    var color: Color {
        switch latency.latencyType {
        case .fast: Colors.green
        case .normal: Colors.orange
        case .slow: Colors.red
        }
    }

    var background: Color {
        color.opacity(.light)
    }

    var value: Int {
        Int(latency.value)
    }
}
