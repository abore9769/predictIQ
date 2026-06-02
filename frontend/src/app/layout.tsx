import type { ReactNode } from 'react';
import { ErrorBoundary } from '../components/ErrorBoundary';
import { darkModeInitScript } from '../lib/darkMode';
import '../styles/accessibility.css';

export const metadata = { title: 'PredictIQ' };

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: darkModeInitScript }} />
      </head>
      <body>
        <ErrorBoundary section="main">
          {children}
        </ErrorBoundary>
      </body>
    </html>
  );
}
