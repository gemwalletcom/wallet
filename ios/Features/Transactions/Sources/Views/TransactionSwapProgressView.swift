// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSwapProgressMarker
import struct Gemstone.GemSwapProgressState
import Style
import SwiftUI

struct TransactionSwapProgressView: View {
    let model: TransactionSwapProgressItemModel

    var body: some View {
        HStack(alignment: .top, spacing: .space12) {
            timelineView
            VStack(alignment: .leading, spacing: .space8) {
                stepContent(model.transfer)
                stepContent(model.swap)
            }
        }
        .padding(.medium)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Colors.listStyleColor)
        .cornerRadius(.space12)
        .cleanListRow()
    }

    private var timelineView: some View {
        VStack(spacing: .zero) {
            marker(for: model.transfer.state)
            connector(color: model.transfer.state.lineColor)
            marker(for: model.swap.state)
        }
        .frame(width: Sizing.list.settings)
    }

    private func stepContent(_ step: TransactionSwapProgressItemModel.Step) -> some View {
        VStack(alignment: .leading, spacing: .space6) {
            HStack(alignment: .center, spacing: .space8) {
                Text(step.title)
                    .font(.app.body)
                    .foregroundStyle(Colors.black)
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)

                Spacer(minLength: .space8)

                statusTag(for: step.state)
            }

            HStack(alignment: .firstTextBaseline, spacing: .space8) {
                Text(step.subtitle)
                    .font(.app.callout)
                    .foregroundStyle(Colors.gray)
                    .lineLimit(2)

                Spacer(minLength: .space8)

                if step.state.marker == .spinner, let estimatedTime = model.estimatedTime {
                    Text(estimatedTime)
                        .font(.app.callout)
                        .foregroundStyle(Colors.gray)
                        .lineLimit(1)
                }
            }
        }
    }

    private func connector(color: Color) -> some View {
        Rectangle()
            .fill(color)
            .frame(width: 1.5, height: Sizing.list.settings)
    }

    private func marker(for state: GemSwapProgressState) -> some View {
        ZStack {
            Circle()
                .stroke(state.color, lineWidth: .space1)
                .background(Circle().fill(state.markerBackground))

            switch state.marker {
            case .spinner:
                LoadingView(size: .small, tint: state.color)
            case .check, .dots, .cross, .swap:
                state.marker.image?
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(state.color)
            }
        }
        .frame(width: Sizing.list.settings, height: Sizing.list.settings)
    }

    @ViewBuilder
    private func statusTag(for state: GemSwapProgressState) -> some View {
        if let tagTitle = state.step.tagTitle {
            Text(tagTitle)
                .font(.app.footnote)
                .foregroundStyle(state.color)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
                .padding(.horizontal, .small)
                .padding(.vertical, .extraSmall)
                .background(state.background)
                .cornerRadius(.space6)
        }
    }
}
