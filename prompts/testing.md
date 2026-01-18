Please generate me a methode to test my programm

Unit Tests

cargo test

This runs 5 tests covering JWT and password hashing.

Manual API Testing

Once the app is running (cargo run), test the endpoints:

1. Health check:                                                                                                                                                                                                                  
   curl http://localhost:8080/health

2. Sign in (get JWT token):                                                                                                                                                                                                       
   curl -X POST http://localhost:8080/auth/signin \                                                                                                                                                                                  
   -H "Content-Type: application/json" \                                                                                                                                                                                           
   -d '{"email": "admin@admin.com", "password": "YOUR_PASSWORD"}'

3. Verify token:                                                                                                                                                                                                                  
   curl -X POST http://localhost:8080/auth/verify \                                                                                                                                                                                  
   -H "Content-Type: application/json" \                                                                                                                                                                                           
   -d '{"token": "YOUR_JWT_TOKEN_HERE"}'

4. Register new user (requires admin token):                                                                                                                                                                                      
   curl -X POST http://localhost:8080/auth/admin/register \                                                                                                                                                                          
   -H "Content-Type: application/json" \                                                                                                                                                                                           
   -H "Authorization: Bearer YOUR_JWT_TOKEN" \                                                                                                                                                                                     
   -d '{"name": "newuser", "email": "user@example.com", "password": "securepassword123"}'

Docker Testing

docker-compose up -d --build                                                                                                                                                                                                      
curl http://localhost:8080/health

Note: For step 2, you need to know the password that was hashed in initial_admin.json. The hash there corresponds to whatever password was used when generating it with the hash_password utility.                                
                     