import React from 'react';
import { render, screen } from '@testing-library/react';
import App from './App';

test('renders HermesFlow header', () => {
  render(<App />);
  const headerElement = screen.getByText(/HermesFlow/i);
  expect(headerElement).toBeInTheDocument();
});

test('renders health status', () => {
  render(<App />);
  const statusElement = screen.getByText(/Status: healthy/i);
  expect(statusElement).toBeInTheDocument();
});

test('displays service name', () => {
  render(<App />);
  const serviceElement = screen.getByText(/Service: frontend/i);
  expect(serviceElement).toBeInTheDocument();
});
