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
            marker(for: model.transfer.status)
            connector(color: model.transfer.status.lineColor)
            marker(for: model.swap.status)
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

                statusTag(for: step.status)
            }

            Text(step.subtitle)
                .font(.app.callout)
                .foregroundStyle(Colors.gray)
                .lineLimit(2)
        }
    }

    private func connector(color: Color) -> some View {
        Rectangle()
            .fill(color)
            .frame(width: 1.5, height: Sizing.list.settings)
    }

    private func marker(for status: TransactionSwapProgressItemModel.Step.Status) -> some View {
        ZStack {
            Circle()
                .stroke(status.color, lineWidth: .space1)
                .background(Circle().fill(status.markerBackground))

            switch status {
            case .completed:
                Images.System.checkmark
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(status.color)
            case .pending:
                LoadingView(size: .small, tint: status.color)
            case .waiting:
                Images.System.ellipsis
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(status.color)
            case .failed:
                Images.System.xmark
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(status.color)
            case .reverted:
                Images.System.xmark
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(status.color)
            case .refunded:
                Images.System.arrowSwap
                    .font(.app.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(status.color)
            }
        }
        .frame(width: Sizing.list.settings, height: Sizing.list.settings)
    }

    @ViewBuilder
    private func statusTag(for status: TransactionSwapProgressItemModel.Step.Status) -> some View {
        if let tagTitle = status.tagTitle {
            Text(tagTitle)
                .font(.app.footnote)
                .foregroundStyle(status.color)
                .lineLimit(1)
                .minimumScaleFactor(0.8)
                .padding(.horizontal, .small)
                .padding(.vertical, .extraSmall)
                .background(status.background)
                .cornerRadius(.space6)
        }
    }
}
