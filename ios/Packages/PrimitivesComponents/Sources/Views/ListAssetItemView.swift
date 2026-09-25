// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetItemRow
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public enum ListAssetItemAction {
    case switcher(enabled: Bool)
    case copy
}

public struct ListAssetItemView: View {
    private let row: GemAssetItemRow
    private let isPrivacyEnabled: Binding<Bool>
    private let action: ((ListAssetItemAction) -> Void)?
    @State private var toggleValue: Bool

    public init(
        row: GemAssetItemRow,
        isPrivacyEnabled: Binding<Bool> = .constant(false),
        action: ((ListAssetItemAction) -> Void)? = nil,
    ) {
        self.row = row
        self.isPrivacyEnabled = row.masksBalance ? isPrivacyEnabled : .constant(false)
        self.action = action
        _toggleValue = State(wrappedValue: row.isToggleOn)
    }

    public var body: some View {
        ListItemFlexibleView(
            left: { AssetImageView(assetImage: AssetImage(icon: row.icon)) },
            primary: { primaryContent },
            secondary: { secondaryContent },
        )
        .onChange(of: toggleValue) { _, newValue in
            action?(.switcher(enabled: newValue))
        }
    }
}

// MARK: - Components

extension ListAssetItemView {
    private var primaryContent: some View {
        VStack(alignment: .leading, spacing: .tiny) {
            HStack(spacing: .tiny) {
                Text(row.title)
                    .textStyle(TextStyle(font: .body, color: .primary, fontWeight: .semibold))
                    .lineLimit(1)
                if let titleExtra = row.titleExtra {
                    Text(titleExtra)
                        .textStyle(.calloutSecondary)
                }
            }
            subtitleView
        }
    }

    @ViewBuilder
    private var subtitleView: some View {
        let texts = [row.subtitle, row.subtitleExtra].compactMap(\.self)
        if texts.isNotEmpty {
            HStack(spacing: .extraSmall) {
                ForEach(Array(texts.enumerated()), id: \.offset) { _, text in
                    Text(text.text.text)
                        .textStyle(TextStyle(font: .footnote, color: text.tone.color))
                }
            }
            .numericTransition(for: texts.map(\.text.text))
        }
    }

    @ViewBuilder
    private var secondaryContent: some View {
        switch row.trailing {
        case let .value(value, extra):
            VStack(alignment: .trailing, spacing: .tiny) {
                PrivacyText(value.text.text, isEnabled: isPrivacyEnabled)
                    .textStyle(TextStyle(font: .callout, color: value.tone.color, fontWeight: .semibold))
                    .numericTransition(for: value.text.text)
                if let extra {
                    PrivacyText(extra.text.text, isEnabled: isPrivacyEnabled)
                        .textStyle(TextStyle(font: .footnote, color: extra.tone.color))
                        .numericTransition(for: extra.text.text)
                }
            }
            .lineLimit(1)
        case .toggle:
            Toggle("", isOn: $toggleValue)
                .labelsHidden()
                .toggleStyle(AppToggleStyle())
        case .copy:
            ListButton(
                image: Images.System.copy,
                padding: .small,
                action: { action?(.copy) },
            )
            .background(Colors.grayVeryLight)
            .foregroundStyle(Colors.gray)
            .clipShape(RoundedRectangle(cornerRadius: 24))
        case .none:
            EmptyView()
        }
    }
}

private extension GemAssetItemRow {
    var isToggleOn: Bool {
        switch trailing {
        case let .toggle(isOn): isOn
        case .value, .copy, .none: false
        }
    }
}
