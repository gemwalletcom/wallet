// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemChartHeader
import GemstonePrimitives
import Style
import SwiftUI

public struct ChartHeaderView: View {
    private let header: GemChartHeader
    private let date: String?

    public init(header: GemChartHeader, date: String? = nil) {
        self.header = header
        self.date = date
    }

    public var body: some View {
        VStack(spacing: Spacing.tiny) {
            if let secondaryValue = header.secondaryValue?.text() {
                Text(secondaryValue)
                    .font(.app.largeTitle)
                    .foregroundStyle(Colors.black)
                    .numericTransition(for: secondaryValue)
                    .minimumScaleFactor(0.5)
                    .lineLimit(1)
                    .padding(.bottom, Spacing.space10)
            }

            HStack(alignment: .center, spacing: Spacing.tiny) {
                let value = header.value.text()
                Text(value)
                    .font(valueFont)
                    .foregroundStyle(header.value.tone.color)
                    .numericTransition(for: value)

                if let change = header.change?.text() {
                    Text(change)
                        .font(changeFont)
                        .foregroundStyle(header.change?.tone.color ?? Colors.gray)
                        .numericTransition(for: change)
                }
            }

            HStack {
                if let date {
                    Text(date)
                        .font(.footnote)
                        .foregroundStyle(Colors.gray)
                }
            }.frame(height: .space16)
        }
    }

    private var valueFont: Font {
        header.secondaryValue != nil ? .app.headline : .title2
    }

    private var changeFont: Font {
        header.secondaryValue != nil ? .app.headline : .callout
    }
}
