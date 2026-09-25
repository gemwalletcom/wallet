// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

public extension View {
    func copyToast(
        model: CopyTypeViewModel,
        isPresenting: Binding<Bool>,
        feedbackGenerator: UINotificationFeedbackGenerator = UINotificationFeedbackGenerator(),
    ) -> some View {
        toast(isPresenting: isPresenting, message: ToastMessage(title: model.message, image: model.systemImage))
            .onChange(of: isPresenting.wrappedValue, initial: true) { _, newValue in
                if newValue {
                    model.copy()
                    feedbackGenerator.notificationOccurred(.success)
                }
            }
    }

    func copyToast(
        _ model: Binding<CopyTypeViewModel?>,
        feedbackGenerator: UINotificationFeedbackGenerator = UINotificationFeedbackGenerator(),
    ) -> some View {
        toast(message: Binding(
            get: { model.wrappedValue.map { ToastMessage(title: $0.message, image: $0.systemImage) } },
            set: { message in
                if message == nil {
                    model.wrappedValue = nil
                }
            },
        ))
        .onChange(of: model.wrappedValue) { _, newValue in
            guard let newValue else { return }
            newValue.copy()
            feedbackGenerator.notificationOccurred(.success)
        }
    }
}
