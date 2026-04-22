// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct ChartBounds {
    static let desiredTickCount = 5

    let minPrice: Double
    let maxPrice: Double
    let visibleLines: [ChartLineViewModel]

    init(candles: [ChartCandleStick], lines: [ChartLineViewModel]) {
        let candleMin = candles.map(\.low).min() ?? 0
        let candleMax = candles.map(\.high).max() ?? 1
        let candleRange = candleMax - candleMin

        let visibleLines = lines.filter {
            $0.price >= candleMin - candleRange * 0.5 && $0.price <= candleMax + candleRange * 0.5
        }

        let overlayMin = visibleLines.map(\.price).min() ?? candleMin
        let overlayMax = visibleLines.map(\.price).max() ?? candleMax
        let priceRange = max(candleMax, overlayMax) - min(candleMin, overlayMin)
        let padding = priceRange * 0.05

        minPrice = min(candleMin, overlayMin) - padding
        maxPrice = max(candleMax, overlayMax) + padding
        self.visibleLines = visibleLines.sorted { $0.price < $1.price }
    }

    var axisTicks: [Double] {
        guard axisStep > 0 else { return [] }
        let first = (minPrice / axisStep).rounded(.up) * axisStep
        return stride(from: first, through: maxPrice, by: axisStep).map { $0 }
    }

    var axisFractionLength: Int {
        let stepDecimals = axisStep > 0 ? max(0, Int(ceil(-log10(axisStep)))) : 0
        let magnitudeDecimals: Int = switch maxPrice {
        case _ where maxPrice >= 1000: 0
        case _ where maxPrice >= 0.01: 2
        case _ where maxPrice >= 0.001: 3
        case _ where maxPrice >= 0.0001: 4
        default: 5
        }
        return min(max(stepDecimals, magnitudeDecimals), 8)
    }

    var axisFormat: FloatingPointFormatStyle<Double> {
        .number.precision(.fractionLength(axisFractionLength))
    }

    private var axisStep: Double {
        let rough = (maxPrice - minPrice) / Double(ChartBounds.desiredTickCount)
        guard rough > 0 else { return 0 }
        let magnitude = pow(10, floor(log10(rough)))
        switch rough / magnitude {
        case ..<1.5: return magnitude
        case ..<2.25: return magnitude * 2
        case ..<3.75: return magnitude * 2.5
        case ..<7.5: return magnitude * 5
        default: return magnitude * 10
        }
    }
}
