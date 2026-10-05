// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.ChartCandleStick
import struct Gemstone.GemFormattedNumber
import PrimitivesComponents
import Style
import SwiftUI

struct CandlestickCanvas: View {
    private enum Metrics {
        static let labelOverlapSpacing: CGFloat = 115
        static let lineStyle = StrokeStyle(lineWidth: 1, dash: [4, 3])
        static let currentPriceStyle = StrokeStyle(lineWidth: 1, dash: [2, 3])
        static let selectionStyle = StrokeStyle(lineWidth: .space1, dash: [5])
        static let cornerRatio: CGFloat = 0.15
        static let axisTickLength: CGFloat = 4
        static let selectionDotSize: CGFloat = .space12
    }

    let plot: CandlestickPlot
    let selectedCandle: ChartCandleStick?

    private let dateFormatter = ChartDateFormatter()

    var body: some View {
        Canvas { context, _ in
            let levels = plot.chart.layout.levels
            drawGrid(levels, in: &context)
            drawVolumes(in: &context)
            drawCandles(in: &context)
            drawLines(in: &context)
            drawSelection(in: &context)
            drawPriceLabels(levels, in: &context)
            drawTimeLabels(in: &context)
        }
    }
}

// MARK: - Drawing

extension CandlestickCanvas {
    private func drawGrid(_ levels: [GemFormattedNumber], in context: inout GraphicsContext) {
        let frame = plot.frame
        for level in levels {
            let y = plot.y(for: level.value)
            context.stroke(line(from: CGPoint(x: frame.minX, y: y), to: CGPoint(x: frame.maxX, y: y)), with: .color(ChartGridStyle.color), style: ChartGridStyle.strokeStyle)
            context.stroke(line(from: CGPoint(x: frame.maxX, y: y), to: CGPoint(x: frame.maxX + Metrics.axisTickLength, y: y)), with: .color(ChartGridStyle.color), lineWidth: ChartGridStyle.lineWidth)
        }
        for tick in plot.chart.xTicks {
            let x = plot.x(for: tick.date)
            context.stroke(line(from: CGPoint(x: x, y: frame.minY), to: CGPoint(x: x, y: frame.maxY)), with: .color(ChartGridStyle.color), style: ChartGridStyle.strokeStyle)
        }
        if plot.showsCurrentPrice {
            let y = plot.y(for: plot.chart.layout.currentPrice.value)
            context.stroke(line(from: CGPoint(x: frame.minX, y: y), to: CGPoint(x: frame.maxX, y: y)), with: .color(Colors.gray.opacity(.semiStrong)), style: Metrics.currentPriceStyle)
        }
    }

    private func drawVolumes(in context: inout GraphicsContext) {
        guard plot.hasVolume else { return }
        let bars = plot.chart.candles.filter { $0.volume > 0 }.map(plot.volume(of:)).reduce(into: Path()) { path, bar in
            path.addRoundedRect(in: bar, cornerSize: cornerSize(of: bar))
        }
        context.drawLayer { layer in
            layer.clip(to: Path(plot.frame))
            layer.fill(bars, with: .color(Colors.gray.opacity(.opacity25)))
        }
    }

    private func drawCandles(in context: inout GraphicsContext) {
        let paths = Dictionary(grouping: zip(plot.chart.candles, plot.chart.layout.tones), by: \.1).mapValues { pairs in
            pairs.map(\.0).reduce(into: Path()) { path, candle in
                let wick = plot.wick(of: candle)
                let body = plot.body(of: candle)
                path.addRoundedRect(in: wick, cornerSize: CGSize(width: wick.width / 2, height: wick.width / 2))
                path.addRoundedRect(in: body, cornerSize: cornerSize(of: body))
            }
        }
        context.drawLayer { layer in
            layer.clip(to: Path(plot.frame))
            for (tone, path) in paths {
                layer.fill(path, with: .color(tone.color))
            }
        }
    }

    private func drawLines(in context: inout GraphicsContext) {
        for line in plot.chart.layout.lines {
            let y = plot.y(for: line.price.value)
            context.stroke(self.line(from: CGPoint(x: plot.frame.minX, y: y), to: CGPoint(x: plot.frame.maxX, y: y)), with: .color(line.kind.color.opacity(.semiStrong)), style: Metrics.lineStyle)
            let label = context.resolve(Text(line.label.text).font(.app.caption).foregroundStyle(Colors.whiteSolid))
            let size = label.measure(in: plot.frame.size)
            let badge = CGRect(x: plot.frame.minX + CGFloat(line.overlapLevel) * Metrics.labelOverlapSpacing, y: y - size.height / 2 - .tiny, width: size.width + .tiny * 2, height: size.height + .tiny * 2)
            context.fill(Path(roundedRect: badge, cornerRadius: .tiny), with: .color(line.kind.color))
            context.draw(label, at: CGPoint(x: badge.midX, y: badge.midY), anchor: .center)
        }
    }

    private func drawSelection(in context: inout GraphicsContext) {
        guard let selectedCandle else { return }
        let x = plot.x(for: selectedCandle.date)
        context.stroke(line(from: CGPoint(x: x, y: plot.frame.minY), to: CGPoint(x: x, y: plot.frame.maxY)), with: .color(Colors.blue), style: Metrics.selectionStyle)
        let dot = CGRect(x: x - Metrics.selectionDotSize / 2, y: plot.y(for: selectedCandle.close) - Metrics.selectionDotSize / 2, width: Metrics.selectionDotSize, height: Metrics.selectionDotSize)
        context.fill(Path(ellipseIn: dot), with: .color(Colors.white))
        context.stroke(Path(ellipseIn: dot.insetBy(dx: .space1, dy: .space1)), with: .color(Colors.blue), lineWidth: .space2)
    }

    private func drawPriceLabels(_ levels: [GemFormattedNumber], in context: inout GraphicsContext) {
        let labelX = plot.frame.maxX + Metrics.axisTickLength + .extraSmall
        for level in levels {
            let label = context.resolve(Text(level.text()).font(.caption2).monospacedDigit().foregroundStyle(Colors.gray))
            context.draw(label, at: CGPoint(x: labelX, y: plot.y(for: level.value)), anchor: .leading)
        }
        guard plot.showsCurrentPrice else { return }
        let currentPrice = plot.chart.layout.currentPrice
        let label = context.resolve(Text(currentPrice.text()).font(.caption2).monospacedDigit().foregroundStyle(Colors.whiteSolid))
        let size = label.measure(in: plot.frame.size)
        let pill = CGRect(x: labelX - .extraSmall, y: plot.y(for: currentPrice.value) - size.height / 2 - .space1, width: size.width + .extraSmall * 2, height: size.height + .space1 * 2)
        context.fill(Path(roundedRect: pill, cornerRadius: Spacing.tiny), with: .color(plot.chart.layout.currentTone.color))
        context.draw(label, at: CGPoint(x: pill.midX, y: pill.midY), anchor: .center)
    }

    private func drawTimeLabels(in context: inout GraphicsContext) {
        for tick in plot.chart.xTicks {
            let label = context.resolve(Text(dateFormatter.string(for: tick)).font(.caption2).foregroundStyle(Colors.gray))
            context.draw(label, at: CGPoint(x: plot.x(for: tick.date), y: plot.frame.maxY + Spacing.small), anchor: .top)
        }
    }

    private func cornerSize(of rect: CGRect) -> CGSize {
        let radius = min(plot.bodyWidth * Metrics.cornerRatio, rect.height / 2)
        return CGSize(width: radius, height: radius)
    }

    private func line(from start: CGPoint, to end: CGPoint) -> Path {
        Path { path in
            path.move(to: start)
            path.addLine(to: end)
        }
    }
}
