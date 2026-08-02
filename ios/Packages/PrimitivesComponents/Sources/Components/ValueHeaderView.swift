// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct ValueHeaderViewSpacing: Sendable {
    public static let standard = ValueHeaderViewSpacing(
        assetImageBottom: .space12,
        titleBottom: .space10,
        subtitleTop: .zero,
        subtitleBottom: .space10,
    )
    public static let transactionAmount = ValueHeaderViewSpacing(
        assetImageBottom: .space8,
        titleBottom: .space6,
        subtitleTop: .space2,
        subtitleBottom: .zero,
    )

    let assetImageBottom: CGFloat
    let titleBottom: CGFloat
    let subtitleTop: CGFloat
    let subtitleBottom: CGFloat

    public init(
        assetImageBottom: CGFloat,
        titleBottom: CGFloat,
        subtitleTop: CGFloat = .zero,
        subtitleBottom: CGFloat,
    ) {
        self.assetImageBottom = assetImageBottom
        self.titleBottom = titleBottom
        self.subtitleTop = subtitleTop
        self.subtitleBottom = subtitleBottom
    }

    public init(
        contentBottom: CGFloat,
        subtitleTop: CGFloat = .zero,
        subtitleBottom: CGFloat,
    ) {
        self.init(
            assetImageBottom: contentBottom,
            titleBottom: contentBottom,
            subtitleTop: subtitleTop,
            subtitleBottom: subtitleBottom,
        )
    }
}

public struct ValueHeaderView: View {
    private let model: any ValueHeaderViewModel

    @Binding var isPrivacyEnabled: Bool

    private let titleActionType: HeaderTitleActionType
    private let spacing: ValueHeaderViewSpacing
    private let onHeaderAction: HeaderButtonAction?
    private let onSubtitleAction: VoidAction
    private let onInfoAction: VoidAction

    public init(
        model: any ValueHeaderViewModel,
        isPrivacyEnabled: Binding<Bool>,
        titleActionType: HeaderTitleActionType,
        spacing: ValueHeaderViewSpacing = .standard,
        onHeaderAction: HeaderButtonAction?,
        onSubtitleAction: VoidAction = nil,
        onInfoAction: VoidAction,
    ) {
        self.model = model
        _isPrivacyEnabled = isPrivacyEnabled
        self.titleActionType = titleActionType
        self.spacing = spacing
        self.onHeaderAction = onHeaderAction
        self.onSubtitleAction = onSubtitleAction
        self.onInfoAction = onInfoAction
    }

    public var body: some View {
        VStack(spacing: .zero) {
            if let assetImage = model.assetImage {
                AssetImageView(
                    assetImage: assetImage,
                    size: .image.semiLarge,
                )
                .padding(.bottom, spacing.assetImageBottom)
            }
            titleView
                .numericTransition(for: model.title)
                .minimumScaleFactor(0.5)
                .font(.app.largeTitle)
                .foregroundStyle(Colors.black)
                .lineLimit(1)
                .padding(.bottom, spacing.titleBottom)

            if let subtitle = model.subtitle {
                subtitleView(subtitle)
                    .numericTransition(for: model.subtitle)
                    .padding(.top, spacing.subtitleTop)
                    .padding(.bottom, spacing.subtitleBottom)
            }

            switch model.isWatchWallet {
            case true:
                Button {
                    onInfoAction?()
                } label: {
                    HStack {
                        Images.System.eye

                        Text(Localized.Wallet.Watch.Tooltip.title)
                            .foregroundStyle(Colors.black)
                            .font(.callout)

                        Images.System.info
                            .tint(Colors.black)
                    }
                    .padding()
                    .background(Colors.grayDarkBackground)
                    .cornerRadius(.medium)
                    .padding(.top, .space10)
                }

            case false:
                HeaderButtonsView(buttons: model.buttons, action: onHeaderAction)
                    .padding(.top, .space8)
            }
        }
    }

    @ViewBuilder
    private func subtitleView(_ subtitle: String) -> some View {
        let content = HStack(spacing: Spacing.space6) {
            PrivacyText(
                subtitle,
                isEnabled: $isPrivacyEnabled,
            )
            .font(.app.headline)
            .foregroundStyle(model.subtitleColor)

            if let subtitleImage = model.subtitleImage {
                subtitleImage
                    .font(.footnote)
                    .foregroundStyle(Colors.secondaryText)
            }
        }

        if let onSubtitleAction {
            Button(action: onSubtitleAction) {
                content
            }
        } else {
            content
        }
    }

    @ViewBuilder
    private var titleView: some View {
        switch titleActionType {
        case .privacyToggle:
            PrivacyToggleView(model.title, isEnabled: $isPrivacyEnabled)
        case .privacyMasked:
            PrivacyText(model.title, isEnabled: $isPrivacyEnabled)
        case let .action(action):
            Button(action: action) {
                PrivacyText(model.title, isEnabled: $isPrivacyEnabled)
            }
        case .none:
            Text(model.title)
        }
    }
}

// MARK: - Previews

#Preview {
    let model = WalletHeaderViewModel(
        walletType: .multicoin,
        totalValue: TotalFiatValue(value: 1000, pnlAmount: 50, pnlPercentage: 5.26),
        currencyCode: Currency.usd.rawValue,
        bannerEventsViewModel: HeaderBannerEventViewModel(events: []),
    )

    ValueHeaderView(
        model: model,
        isPrivacyEnabled: .constant(false),
        titleActionType: .privacyToggle,
        onHeaderAction: .none,
        onInfoAction: .none,
    )
}
