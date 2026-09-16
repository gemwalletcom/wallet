// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public protocol CurrencyInputConfigurable {
    var placeholder: String { get }
    var currencySymbol: String { get }
    var currencyPosition: CurrencyTextField.CurrencyPosition { get }
    var secondaryText: String { get }
    var keyboardType: UIKeyboardType { get }
    var actionStyle: CurrencyInputActionStyle? { get }
    var onTapActionButton: (() -> Void)? { get }
    func sanitize(_ text: String) -> String
}

public struct CurrencyInputView: View {
    @Binding var text: String

    private let config: CurrencyInputConfigurable

    public init(text: Binding<String>, config: CurrencyInputConfigurable) {
        _text = text
        self.config = config
    }

    public var body: some View {
        VStack(alignment: .center, spacing: .small) {
            HStack(spacing: .small) {
                if let actionStyle = config.actionStyle, actionStyle.position == .amount {
                    actionButton(for: actionStyle.position)
                }

                CurrencyTextField(
                    config.placeholder,
                    text: $text,
                    currencySymbol: config.currencySymbol,
                    currencyPosition: config.currencyPosition,
                    keyboardType: config.keyboardType,
                )
                .onChange(of: text) { _, newValue in
                    text = config.sanitize(newValue)
                }
            }

            if let actionStyle = config.actionStyle, actionStyle.position == .secondary {
                actionButton(for: actionStyle.position)
            } else {
                secondaryTextView
            }
        }
        .onAppear {
            UITextField.appearance().clearButtonMode = .never
        }
    }

    func actionButton(for position: CurrencyInputActionPosition) -> some View {
        Button {
            config.onTapActionButton?()
        } label: {
            switch position {
            case .amount:
                actionImage
            case .secondary:
                HStack {
                    secondaryTextView
                    actionImage
                }
            }
        }
        .buttonStyle(.plain)
        .disabled(config.actionStyle == nil)
    }

    @ViewBuilder
    var actionImage: some View {
        if let actionStyle = config.actionStyle {
            actionStyle.image
                .resizable()
                .frame(width: actionStyle.imageSize, height: actionStyle.imageSize)
                .foregroundStyle(Colors.gray)
        }
    }

    var secondaryTextView: some View {
        Text(config.secondaryText)
            .textStyle(.calloutSecondary.weight(.medium))
            .frame(minHeight: .list.image)
    }
}
