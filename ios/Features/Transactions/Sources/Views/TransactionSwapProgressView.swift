// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
            marker(for: model.transfer)
            connector(color: model.transfer.lineColor)
            marker(for: model.swap)
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

                statusTag(for: step)
            }

            HStack(alignment: .firstTextBaseline, spacing: .space8) {
                Text(step.subtitle)
                    .font(.app.callout)
                    .foregroundStyle(Colors.gray)
                    .lineLimit(2)

                Spacer(minLength: .space8)

                if step.showsSpinner, let estimatedTime = model.estimatedTime {
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

    private func marker(for step: TransactionSwapProgressItemModel.Step) -> some View {
        ZStack {
            Circle()
                .stroke(step.color, lineWidth: .space1)
                .background(Circle().fill(step.markerBackground))
            if step.showsSpinner {
                LoadingView(size: .small, tint: step.color)
            } else {
                step.markerImage?
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(step.color)
            }
        }
        .frame(width: Sizing.list.settings, height: Sizing.list.settings)
    }

    @ViewBuilder
    private func statusTag(for step: TransactionSwapProgressItemModel.Step) -> some View {
        if let tagTitle = step.tagTitle {
            Text(tagTitle)
                .font(.app.footnote)
                .foregroundStyle(step.color)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
                .padding(.horizontal, .small)
                .padding(.vertical, .extraSmall)
                .background(step.tagBackground)
                .cornerRadius(.space6)
        }
    }
}
