// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public struct ConnectionStatusBanner: View {
    private let model: ConnectionStatusViewModel
    private let onClose: () -> Void

    public init(model: ConnectionStatusViewModel, onClose: @escaping () -> Void) {
        self.model = model
        self.onClose = onClose
    }

    public var body: some View {
        HStack(spacing: .small) {
            model.icon
                .foregroundStyle(model.iconColor)

            VStack(alignment: .leading, spacing: .extraSmall) {
                if let title = model.title {
                    Text(title)
                        .font(.subheadline.weight(.medium))
                        .foregroundStyle(Colors.black)
                        .lineLimit(1)
                        .minimumScaleFactor(0.85)
                }
                Text(model.subtitle)
                    .font(.footnote)
                    .foregroundStyle(Colors.gray)
                    .lineLimit(2)
            }

            Spacer(minLength: 0)

            closeButton
        }
        .padding(.medium)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background {
            Color.clear
                .liquidGlass(interactive: false, in: RoundedRectangle(cornerRadius: .medium)) {
                    $0
                        .background(Colors.grayVeryLight)
                        .clipShape(RoundedRectangle(cornerRadius: .medium))
                }
        }
        .padding(.horizontal, .medium)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("connectionStatusBanner")
    }

    private var closeButton: some View {
        ZStack {
            Images.System.xmark
                .font(.footnote.weight(.bold))
                .foregroundStyle(Colors.gray)
                .frame(width: Sizing.list.settings, height: Sizing.list.settings)
                .liquidGlass(in: Circle()) {
                    $0.background(Circle().fill(Colors.grayVeryLightFaded))
                }

            Button(action: onClose) {
                Color.clear
                    .frame(width: Sizing.list.settings, height: Sizing.list.settings)
                    .contentShape(Circle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("connectionStatusDismiss")
        }
    }
}
