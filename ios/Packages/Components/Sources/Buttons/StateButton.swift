// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public protocol StateButtonViewable: Sendable {
    var title: String { get }
    var buttonState: ButtonState { get }
    var icon: Image? { get }
    var infoText: String? { get }

    @MainActor func action()
}

public extension StateButtonViewable {
    var infoText: String? {
        nil
    }
}

public struct StateButton: View {
    public static let defaultTextStyle = TextStyle(font: .body.weight(.semibold), color: Colors.whiteSolid)

    public let textValue: TextValue
    public let image: Image?
    public let infoTextValue: TextValue?
    public let state: ButtonState

    private let action: () -> Void

    public init(
        text: String,
        textStyle: TextStyle = StateButton.defaultTextStyle,
        state: ButtonState = .normal,
        image: Image? = nil,
        infoTitle: String? = nil,
        infoTitleStyle: TextStyle = .calloutSecondary,
        truncationMode: Text.TruncationMode = .tail,
        action: @escaping () -> Void,
    ) {
        textValue = TextValue(text: text, style: textStyle, truncationMode: truncationMode)
        self.state = state
        infoTextValue = infoTitle.map { TextValue(text: $0, style: infoTitleStyle) }
        self.action = action
        self.image = image
    }

    public var body: some View {
        VStack(spacing: .tiny) {
            if let infoTextValue {
                Text(infoTextValue.text)
                    .textStyle(infoTextValue.style)
                    .multilineTextAlignment(.center)
            }
            Button(action: action) {
                HStack {
                    if let image {
                        image
                            .foregroundStyle(textValue.style.color)
                    }
                    Text(textValue.text)
                        .foregroundStyle(textValue.style.color)
                        .truncationMode(textValue.truncationMode)
                        .multilineTextAlignment(.center)
                }
                .font(textValue.style.font)
            }
            .buttonStyle(.primary(state))
            .disabled(state.isDisabled)
            .accessibilityIdentifier("main_action")
        }
    }
}

public extension StateButton {
    init(_ model: StateButtonViewable) {
        self.init(
            text: model.title,
            state: model.buttonState,
            image: model.icon,
            infoTitle: model.infoText,
            action: model.action,
        )
    }
}

public extension ButtonState {
    init(
        _ viewState: StateViewType<some Any>,
        showProgress: Bool = true,
        isDisabled: Bool? = nil,
    ) {
        if let isDisabled, isDisabled {
            self = .disabled
            return
        }
        switch viewState {
        case .loading: self = .loading(showProgress: showProgress)
        case .noData: self = .disabled
        case .data: self = .normal
        case .error:
            if let isDisabled, !isDisabled {
                self = .normal
                return
            }
            self = .disabled
        }
    }
}

// MARK: - Previews

#Preview {
    List {
        Section(header: Text("Primary · normal")) {
            StateButton(text: "Submit",
                        state: .normal,
                        action: {})
            StateButton(text: "Submit",
                        state: .normal,
                        image: Images.System.settings,
                        action: {})
        }

        Section(header: Text("Primary · normal + info")) {
            StateButton(text: "Submit",
                        state: .normal,
                        infoTitle: "Approve token",
                        action: {})
            StateButton(text: "Submit",
                        state: .normal,
                        image: Images.System.settings,
                        infoTitle: "Long info title Long info title Long info title",
                        action: {})
        }

        Section(header: Text("Primary · loading")) {
            StateButton(text: "Submit",
                        state: .loading(),
                        image: Images.System.settings,
                        action: {})
        }

        Section(header: Text("Primary · disabled")) {
            StateButton(text: "Submit",
                        state: .normal,
                        image: Images.System.settings,
                        action: {})
                .disabled(true)

            StateButton(text: "Submit",
                        state: .disabled,
                        action: {})
        }
    }
    .padding()
}
