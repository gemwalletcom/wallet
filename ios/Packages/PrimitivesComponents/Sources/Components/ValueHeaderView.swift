// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.formattedCurrency
import func Gemstone.formattedPercentage
import struct Gemstone.GemWalletHomeViewState
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
            titleLine
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

                        Spacer(minLength: .zero)

                        Text(Localized.Wallet.Watch.Tooltip.title)
                            .foregroundStyle(Colors.black)
                            .font(.callout)
                            .multilineTextAlignment(.center)
                            .fixedSize(horizontal: false, vertical: true)

                        Spacer(minLength: .zero)

                        Images.System.info
                            .tint(Colors.black)
                    }
                    .padding()
                    .frame(maxWidth: .infinity)
                    .background(Colors.listStyleColor)
                    .cornerRadius(.medium)
                    .padding(.top, .space8)
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

    private var titleLine: some View {
        Text("\u{00a0}")
            .font(.app.largeTitle)
            .lineLimit(1)
            .hidden()
            .accessibilityHidden(true)
            .frame(maxWidth: .infinity)
            .overlay {
                titleView
                    .numericTransition(for: model.title)
                    .minimumScaleFactor(0.35)
                    .font(.app.largeTitle)
                    .foregroundStyle(Colors.black)
                    .lineLimit(1)
                    .frame(maxWidth: .infinity)
            }
    }

    @ViewBuilder
    private var titleView: some View {
        switch titleActionType {
        case .privacyToggle:
            PrivacyToggleView(model.title, isEnabled: $isPrivacyEnabled)
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
    let amount = {
        var amount = formattedCurrency(value: 50, code: Currency.usd.rawValue, style: .fiat)
        amount.notation = .signed
        return amount
    }()
    let model = WalletHeaderViewModel(state: GemWalletHomeViewState(
        total: formattedCurrency(value: 1000, code: Currency.usd.rawValue, style: .fiat),
        pnl: .pnl(
            amount: amount,
            percent: formattedPercentage(value: 5.26, style: .unsigned),
        ),
        pnlTone: .positive,
        headerActions: .buttons(buttons: []),
        showCollections: false,
        showsPerpetuals: false,
        visibleBanners: [],
    ))

    ValueHeaderView(
        model: model,
        isPrivacyEnabled: .constant(false),
        titleActionType: .privacyToggle,
        onHeaderAction: .none,
        onInfoAction: .none,
    )
}
