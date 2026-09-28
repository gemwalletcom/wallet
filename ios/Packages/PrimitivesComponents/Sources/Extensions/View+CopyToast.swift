// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCopy
import Style
import SwiftUI

public extension View {
    func copyToast(
        copy: GemCopy,
        isPresenting: Binding<Bool>,
        feedbackGenerator: UINotificationFeedbackGenerator = UINotificationFeedbackGenerator(),
    ) -> some View {
        toast(isPresenting: isPresenting, message: ToastMessage(title: copy.copiedMessage, image: SystemImage.copy))
            .onChange(of: isPresenting.wrappedValue, initial: true) { _, newValue in
                if newValue {
                    Clipboard.copy(copy)
                    feedbackGenerator.notificationOccurred(.success)
                }
            }
    }

    func copyToast(
        _ copy: Binding<GemCopy?>,
        feedbackGenerator: UINotificationFeedbackGenerator = UINotificationFeedbackGenerator(),
    ) -> some View {
        toast(message: Binding(
            get: { copy.wrappedValue.map { ToastMessage(title: $0.copiedMessage, image: SystemImage.copy) } },
            set: { message in
                if message == nil {
                    copy.wrappedValue = nil
                }
            },
        ))
        .onChange(of: copy.wrappedValue) { _, newValue in
            guard let newValue else { return }
            Clipboard.copy(newValue)
            feedbackGenerator.notificationOccurred(.success)
        }
    }
}
