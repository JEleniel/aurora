/// Error logging service for centralized error handling
import { invoke } from '@tauri-apps/api/core';

export type ErrorLevel = 'error' | 'warn' | 'info' | 'debug';

export interface ErrorLog {
	level: ErrorLevel;
	message: string;
	context?: string;
	timestamp: Date;
}

class ErrorLoggingService {
	private errorHistory: ErrorLog[] = [];
	private readonly maxHistorySize = 100;

	/**
	 * Log an error locally and send to backend
	 */
	async logError(level: ErrorLevel, message: string, context?: string): Promise<void> {
		const errorLog: ErrorLog = {
			level,
			message,
			context,
			timestamp: new Date(),
		};

		// Store locally
		this.errorHistory.push(errorLog);
		if (this.errorHistory.length > this.maxHistorySize) {
			this.errorHistory.shift();
		}

		// Log to browser console
		const consoleMethod = level === 'error' ? 'error' : level === 'warn' ? 'warn' : 'log';
		console[consoleMethod](`[${errorLog.timestamp.toISOString()}] [${context || 'app'}] ${message}`);

		// Send to backend for persistent logging
		try {
			await invoke('log_error', {
				level,
				message,
				context,
			});
		} catch (backendError) {
			// If backend logging fails, at least we have it in console and local history
			console.warn('Failed to send error to backend:', backendError);
		}
	}

	/**
	 * Log an error with full context
	 */
	async error(message: string, context?: string, error?: Error): Promise<void> {
		const fullMessage = error ? `${message}: ${error.message}` : message;
		await this.logError('error', fullMessage, context);
	}

	/**
	 * Log a warning
	 */
	async warn(message: string, context?: string): Promise<void> {
		await this.logError('warn', message, context);
	}

	/**
	 * Log info message
	 */
	async info(message: string, context?: string): Promise<void> {
		await this.logError('info', message, context);
	}

	/**
	 * Log debug message
	 */
	async debug(message: string, context?: string): Promise<void> {
		await this.logError('debug', message, context);
	}

	/**
	 * Get error history
	 */
	getHistory(): ErrorLog[] {
		return [...this.errorHistory];
	}

	/**
	 * Clear error history
	 */
	clearHistory(): void {
		this.errorHistory = [];
	}

	/**
	 * Get errors by level
	 */
	getErrorsByLevel(level: ErrorLevel): ErrorLog[] {
		return this.errorHistory.filter((log) => log.level === level);
	}

	/**
	 * Get recent errors (last N)
	 */
	getRecent(count: number = 10): ErrorLog[] {
		return this.errorHistory.slice(-count);
	}
}

export const errorLogging = new ErrorLoggingService();
