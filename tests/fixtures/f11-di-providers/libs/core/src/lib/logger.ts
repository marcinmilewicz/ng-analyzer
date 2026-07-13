export abstract class Logger {
  abstract log(message: string): void;
}

export class FileLogger extends Logger {
  log(message: string): void {
    console.log(message);
  }
}
