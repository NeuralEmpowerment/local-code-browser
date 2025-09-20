import React from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './ui/App'

const root = document.getElementById('root')!
createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)

