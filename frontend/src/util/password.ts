export function generateRandomPassword(length = 12): string {
    const charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=";
    const array = new Uint32Array(length);
    window.crypto.getRandomValues(array);
    
    let password = "";
    for (let i = 0; i < length; i++) {
        password += charset[array[i] % charset.length];
    }
    return password;
}